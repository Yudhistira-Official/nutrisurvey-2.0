use nutrisurvey_lib::{
    ai, commands,
    export::{self, ExportRequest, FoodEntry, MealTime, NutritionTargets},
    foods, import, meals,
    models::{AiMealInput, AiRequest, RecommendationFilter, TdeeRequest},
    nutrition,
    project::{self, ProjectFile, ProjectFood, ProjectMeal, ProjectTargets},
    storage::Storage,
};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

static IDS: AtomicUsize = AtomicUsize::new(0);

async fn storage() -> (Storage, std::path::PathBuf) {
    let id = IDS.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("nutrisurvey-acceptance-{id}"));
    let path = root.join("app-data/nutrisurvey.sqlite3");
    let _ = tokio::fs::remove_dir_all(&root).await;
    let storage = Storage::open_path(&path).await.unwrap();
    (storage, root)
}

async fn mock_ai(
    body: &'static str,
) -> (
    String,
    std::net::SocketAddr,
    tokio::task::JoinHandle<String>,
) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        let mut buffer = [0; 4096];
        loop {
            let count = socket.read(&mut buffer).await.unwrap();
            if count == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..count]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).await.unwrap();
        String::from_utf8(request).unwrap()
    });
    (
        format!("http://example.com:{}", address.port()),
        address,
        task,
    )
}

fn ai_request(base_url: String) -> AiRequest {
    AiRequest {
        target_tdee: 2000,
        target_carbs: 250,
        target_protein: 100,
        target_fat: 60,
        available_meal_types: vec!["Sarapan".into()],
        provider: "openai".into(),
        model: "acceptance-model".into(),
        api_key: "acceptance-secret".into(),
        base_url,
    }
}

#[tokio::test]
async fn native_acceptance_covers_readiness_import_search_recommendations_and_tdee() {
    let (storage, path) = storage().await;
    assert!(!foods::status(&storage).await.unwrap().is_ready);
    let standard = b"Nama Makanan;Kategori;Energi;Protein\nNasi;Pokok;130;2,7\n";
    let scraper =
        b"makanan,kategori,komponen_nutrient_1,isi_nutrient_1\nTelur,Protein,Protein,13 g\n";
    assert_eq!(
        import::import_csv(&storage, standard, "standard.csv")
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        import::import_csv(&storage, scraper, "scraper.csv")
            .await
            .unwrap(),
        1
    );
    let bundled_resources = [
        ("standard-resource.csv", b"Nama Makanan;Kategori;Energi;Protein\nNasi Seed;Pokok;130;2,7\n".as_slice()),
        ("scraper-resource.csv", b"makanan,kategori,komponen_nutrient_1,isi_nutrient_1\nTelur Seed,Protein,Protein,13 g\n".as_slice()),
    ];
    assert_eq!(
        import::seed_csvs(&storage, &bundled_resources)
            .await
            .unwrap(),
        2
    );
    assert!(import::seed_is_complete(&storage).await.unwrap());
    assert!(foods::status(&storage).await.unwrap().is_ready);
    assert_eq!(
        foods::search(&storage, "Nasi Seed", 20)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(foods::search(&storage, "nAs", 20).await.unwrap().len(), 2);
    let recommendations = foods::recommend(
        &storage,
        &[RecommendationFilter {
            nutrient: "energi".into(),
            operator: "<".into(),
            value: 200.0,
        }],
    )
    .await
    .unwrap();
    assert!(!recommendations.is_empty());
    let tdee = nutrition::calculate_tdee(TdeeRequest {
        weight_kg: 70.0,
        height_cm: 175.0,
        age: 30,
        gender: "male".into(),
        activity_factor: 1.2,
        injury_factor: 1.0,
        is_manual_factors: false,
        bmi_standard: "asia_pacific".into(),
    })
    .unwrap();
    assert_eq!(tdee.total_daily_energy_expenditure, 2035.2);
    let _ = tokio::fs::remove_dir_all(path).await;
}

#[tokio::test]
async fn native_acceptance_seeds_configured_resource_dir_from_real_csv_files() {
    let id = IDS.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("nutrisurvey-resource-acceptance-{id}"));
    let data_dir = root.join("app-data");
    let resource_dir = root.join("resources");
    tokio::fs::create_dir_all(&resource_dir).await.unwrap();
    tokio::fs::write(
        resource_dir.join("01-standard.csv"),
        b"Nama Makanan;Kategori;Energi\nResource Rice;Pokok;130\n",
    )
    .await
    .unwrap();
    tokio::fs::write(
        resource_dir.join("02-scraper.csv"),
        b"makanan,kategori,komponen_nutrient_1,isi_nutrient_1\nResource Egg,Protein,Protein,13 g\n",
    )
    .await
    .unwrap();
    let database = data_dir.join("nutrisurvey.sqlite3");
    let storage = Storage::open_paths(&database, &resource_dir).await.unwrap();
    assert_eq!(
        nutrisurvey_lib::seed_configured_resources(&storage)
            .await
            .unwrap(),
        2
    );
    assert!(import::seed_is_complete(&storage).await.unwrap());
    assert_eq!(
        foods::search(&storage, "Resource Rice", 20)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        foods::search(&storage, "Resource Egg", 20)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        nutrisurvey_lib::seed_configured_resources(&storage)
            .await
            .unwrap(),
        0
    );
    drop(storage);
    tokio::fs::remove_dir_all(root).await.unwrap();
}

#[tokio::test]
async fn native_acceptance_uses_production_nutri_file_roundtrip() {
    let (storage, root) = storage().await;
    let path = root.join("project.nutri");
    let project = ProjectFile {
        version: 1,
        foods: vec![ProjectFood {
            id: "fixture-1".into(),
            name: "Nasi 🍚".into(),
            serving_size: 100.0,
            serving_unit: "g".into(),
            servings_per_container: 1.0,
            amount: 125.0,
            meal_time: "BREAKFAST".into(),
            nutrients: HashMap::from([(String::from("energi"), 130.0)]),
        }],
        meals: vec![ProjectMeal {
            id: "BREAKFAST".into(),
            label: "Makan Pagi".into(),
        }],
        targets: ProjectTargets {
            kcal: 2000.0,
            carbs: 250.0,
            protein: 100.0,
            fat: 60.0,
        },
    };
    project::save(&path, &project).await.unwrap();
    assert!(path.is_file());
    let loaded = project::load(&path).await.unwrap();
    assert_eq!(loaded, project);
    assert_eq!(loaded.foods[0].name, "Nasi 🍚");
    drop(storage);
    tokio::fs::remove_dir_all(root).await.unwrap();
}

#[tokio::test]
async fn native_acceptance_covers_mocked_ai_meal_mapping_and_secret_redaction() {
    let (storage, path) = storage().await;
    import::import_csv(
        &storage,
        b"Nama Makanan;Energi;Protein\nNasi;130;2,7\n",
        "fixture.csv",
    )
    .await
    .unwrap();
    let body = r#"{"choices":[{"message":{"content":"{\"meal_plan\":[{\"meal_type\":\"Sarapan\",\"food_keyword\":\"Nasi\",\"suggested_grams\":100,\"reasoning\":\"fixture\"}]}"}}]}"#;
    let (base_url, address, task) = mock_ai(body).await;
    let client = Client::builder()
        .resolve("example.com", address)
        .build()
        .unwrap();
    let result = ai::generate_menu_with_client(&storage, ai_request(base_url), &client)
        .await
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].matched_food_name, "Nasi");
    let request = task.await.unwrap();
    assert!(request.contains("Bearer acceptance-secret"));
    assert!(
        !format!("{:?}", ai_request("https://example.test".into())).contains("acceptance-secret")
    );
    let mapped = meals::map_ai_items(
        &storage,
        &[AiMealInput {
            meal_type: "Sarapan".into(),
            food_keyword: "Nasi".into(),
            suggested_grams: 100,
            reasoning: "fixture".into(),
        }],
    )
    .await
    .unwrap();
    assert_eq!(mapped[0].matched_food_name, "Nasi");
    let _ = tokio::fs::remove_dir_all(path).await;
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn assert_tree_has_no_secret(root: &std::path::Path, secret: &str) {
    assert!(root.exists(), "artifact path missing: {}", root.display());
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            for entry in std::fs::read_dir(path).unwrap() {
                pending.push(entry.unwrap().path());
            }
        } else if path.is_file() {
            assert!(!String::from_utf8_lossy(&std::fs::read(path).unwrap()).contains(secret));
        }
    }
}

#[test]
fn native_acceptance_covers_unicode_rtf_and_artifact_secret_scans() {
    let secret = "acceptance-secret";
    let request = ExportRequest {
        foods: vec![FoodEntry {
            meal_time: "BREAKFAST".into(),
            name: "Nasi 🍚".into(),
            amount: 100.0,
            serving_size: 100.0,
            serving_unit: Some("g".into()),
            nutrients: HashMap::from([(String::from("energy"), 130.0)]),
        }],
        meal_times: Some(vec![MealTime {
            id: "BREAKFAST".into(),
            label: "Makan Pagi".into(),
        }]),
        targets: Some(NutritionTargets {
            kcal: 2000.0,
            carbs: 250.0,
            protein: 100.0,
            fat: 60.0,
        }),
    };
    let rtf = export::render_rtf(request, include_bytes!("../../Assets/template.rtf")).unwrap();
    let output = String::from_utf8(rtf).unwrap();
    assert!(output.contains("Nasi \\u-10180?\\u-8358?"));
    assert!(!output.contains("Nasi 🍚"));
    assert_eq!(commands::ping(), "pong");

    let (storage, path) = tokio::runtime::Runtime::new().unwrap().block_on(storage());
    let report_path = path.join("acceptance-report.rtf");
    let database_path = path.join("app-data/nutrisurvey.sqlite3");
    std::fs::write(&report_path, output).unwrap();
    let sqlite = std::fs::read(&database_path).unwrap();
    assert!(!String::from_utf8_lossy(&sqlite).contains(secret));
    assert!(!String::from_utf8_lossy(&std::fs::read(&report_path).unwrap()).contains(secret));
    let root = repo_root();
    assert_tree_has_no_secret(&root.join("out"), secret);
    assert_tree_has_no_secret(&root.join("Assets"), secret);
    assert_tree_has_no_secret(&root.join("DatabaseMakanan"), secret);
    assert!(!"https://example.test/api/v1".contains(secret));
    assert!(!format!("{:?}", ai_request("https://example.test".into())).contains(secret));
    assert!(!"startup log: database ready; export complete".contains(secret));
    drop(storage);
    let _ = std::fs::remove_dir_all(path);
}

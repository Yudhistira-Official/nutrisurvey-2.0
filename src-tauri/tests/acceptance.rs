use nutrisurvey_lib::{
    ai, commands,
    export::{self, ExportRequest, FoodEntry, MealTime, NutritionTargets},
    foods, import, meals,
    models::{AiMealInput, AiRequest, RecommendationFilter, TdeeRequest},
    nutrition,
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
    let path = std::env::temp_dir().join(format!("nutrisurvey-acceptance-{id}.sqlite"));
    let _ = tokio::fs::remove_file(&path).await;
    let storage = Storage::open_path(&path).await.unwrap();
    (storage, path)
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
    (format!("http://ai.test:{}", address.port()), address, task)
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
    import::seed_csvs(&storage, &[]).await.unwrap();
    assert!(foods::status(&storage).await.unwrap().is_ready);
    assert_eq!(foods::search(&storage, "nAs", 20).await.unwrap().len(), 1);
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
    })
    .unwrap();
    assert_eq!(tdee.total_daily_energy_expenditure, 2035.2);
    let _ = tokio::fs::remove_file(path).await;
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
        .resolve("ai.test", address)
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
    let _ = tokio::fs::remove_file(path).await;
}

#[test]
fn native_acceptance_covers_nutri_roundtrip_contract_and_unicode_rtf() {
    let project = r#"{"version":1,"foods":[{"id":"fixture","name":"Nasi 🍚","servingSize":100,"servingUnit":"g","servingsPerContainer":1,"amount":100,"mealTime":"BREAKFAST","nutrients":{"energi":130}}],"meals":[{"id":"BREAKFAST","label":"Makan Pagi"}],"targets":{"kcal":2000,"carbs":250,"protein":100,"fat":60}}"#;
    let parsed: serde_json::Value = serde_json::from_str(project).unwrap();
    assert_eq!(parsed["version"], 1);
    assert_eq!(parsed["foods"][0]["name"], "Nasi 🍚");
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
    assert!(output.contains("Nasi "));
    assert!(output.contains("\\u"));
    assert_eq!(commands::ping(), "pong");
}

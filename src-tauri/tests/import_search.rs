use nutrisurvey_lib::{foods, import, storage::Storage};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static IDS: AtomicU64 = AtomicU64::new(0);

async fn storage() -> (Storage, std::path::PathBuf) {
    let id = IDS.fetch_add(1, Ordering::Relaxed);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir()
        .join(format!("nutrisurvey-task3-{stamp}-{id}"))
        .join("db.sqlite");
    let storage = Storage::open_path(&path).await.unwrap();
    (storage, path)
}

async fn cleanup(path: std::path::PathBuf) {
    let _ = tokio::fs::remove_dir_all(path.parent().unwrap()).await;
}

#[tokio::test]
async fn imports_semicolon_standard_csv_with_serving_and_locale_numbers() {
    let (storage, path) = storage().await;
    let csv = "ID;Nama Makanan;Kategori;Jumlah Sajian;Per Sajian;Energi;Protein\n1;Nasi;Pokok;2;100 g;130 kkal;2,7 g\n2;;Skip;1;100 g;0;0\n";
    assert_eq!(
        import::import_csv(&storage, csv.as_bytes(), "standard.csv")
            .await
            .unwrap(),
        1
    );
    let result = foods::search(&storage, "nAs", 20).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Nasi");
    assert_eq!(result[0].servings_per_container, 2.0);
    assert_eq!(result[0].serving_size, 100.0);
    assert_eq!(result[0].nutrients.get("protein"), Some(&2.7));
    cleanup(path).await;
}

#[tokio::test]
async fn imports_comma_standard_csv_and_skips_malformed_values() {
    let (storage, path) = storage().await;
    let csv = "ID,Nama Makanan,Kategori,Jumlah Sajian,Per Sajian,Energi,Protein\n1,Apel,Buah,1,porsi (150 g),52,not-a-number\n2,Pir,Buah,1,100 g,48,0\n";
    assert_eq!(
        import::import_csv(&storage, csv.as_bytes(), "comma.csv")
            .await
            .unwrap(),
        2
    );
    let result = foods::search(&storage, "", 20).await.unwrap();
    assert_eq!(
        result
            .iter()
            .map(|food| food.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Apel", "Pir"]
    );
    assert_eq!(result[0].serving_size, 150.0);
    assert!(!result[0].nutrients.contains_key("protein"));
    cleanup(path).await;
}

#[tokio::test]
async fn imports_scraper_format_updates_duplicate_food_and_creates_nutrients() {
    let (storage, path) = storage().await;
    let csv = "makanan,kategori,komponen_nutrient_1,isi_nutrient_1,komponen_nutrient_2,isi_nutrient_2\n\"Snack 125g;metadata\",Snack,Protein,\"1,5 g\",Energy,100 kkal\n\"Snack 125g;other\",Snack,Protein,2 g,Energy,-\n";
    assert_eq!(
        import::import_csv(&storage, csv.as_bytes(), "scraper.csv")
            .await
            .unwrap(),
        2
    );
    let result = foods::search(&storage, "snack", 20).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].nutrients.get("protein"), Some(&2.0));
    assert_eq!(result[0].nutrients.get("energi"), Some(&100.0));
    cleanup(path).await;
}

#[tokio::test]
async fn synchronize_sources_skips_unchanged_and_replaces_changed_sources() {
    let (storage, path) = storage().await;
    let first = [("foods.csv", b"Nama Makanan\nRice\n".as_slice())];
    assert_eq!(
        import::synchronize_sources(&storage, &first).await.unwrap(),
        1
    );
    assert_eq!(
        import::synchronize_sources(&storage, &first).await.unwrap(),
        0
    );
    assert_eq!(foods::search(&storage, "rice", 20).await.unwrap().len(), 1);

    let changed = [("foods.csv", b"Nama Makanan\nApple\n".as_slice())];
    assert_eq!(
        import::synchronize_sources(&storage, &changed)
            .await
            .unwrap(),
        1
    );
    assert!(foods::search(&storage, "rice", 20)
        .await
        .unwrap()
        .is_empty());
    assert_eq!(foods::search(&storage, "apple", 20).await.unwrap().len(), 1);
    cleanup(path).await;
}

#[tokio::test]
async fn synchronize_sources_rolls_back_database_and_manifest_on_import_error() {
    let (storage, path) = storage().await;
    let valid = [("foods.csv", b"Nama Makanan\nRice\n".as_slice())];
    import::synchronize_sources(&storage, &valid).await.unwrap();

    let invalid = [("foods.csv", b"Category\nBroken\n".as_slice())];
    assert!(import::synchronize_sources(&storage, &invalid)
        .await
        .is_err());
    assert_eq!(foods::search(&storage, "rice", 20).await.unwrap().len(), 1);
    assert!(foods::search(&storage, "broken", 20)
        .await
        .unwrap()
        .is_empty());
    assert_eq!(
        import::synchronize_sources(&storage, &valid).await.unwrap(),
        0
    );
    cleanup(path).await;
}

#[tokio::test]
async fn search_prioritizes_exact_prefix_then_contains_matches() {
    let (storage, path) = storage().await;
    let csv = "Nama Makanan\nRice Cake\nRice\nBrown Rice\nRice Pudding\n";
    import::import_csv(&storage, csv.as_bytes(), "ranking.csv")
        .await
        .unwrap();
    let result = foods::search(&storage, "rice", 20).await.unwrap();
    assert_eq!(
        result
            .iter()
            .map(|food| food.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Rice", "Rice Cake", "Rice Pudding", "Brown Rice"]
    );
    cleanup(path).await;
}

#[tokio::test]
async fn search_respects_empty_query_and_twenty_result_cap() {
    let (storage, path) = storage().await;
    let mut csv = String::from("Nama Makanan\n");
    for index in (0..25).rev() {
        csv.push_str(&format!("Food {index}\n"));
    }
    import::import_csv(&storage, csv.as_bytes(), "many.csv")
        .await
        .unwrap();
    assert_eq!(foods::search(&storage, "", 100).await.unwrap().len(), 5);
    assert_eq!(
        foods::search(&storage, "food", 100).await.unwrap().len(),
        20
    );
    assert_eq!(foods::search(&storage, "food", 2).await.unwrap().len(), 2);
    cleanup(path).await;
}

#[tokio::test]
async fn malformed_headers_return_import_error_without_partial_rows() {
    let (storage, path) = storage().await;
    let error = import::import_csv(&storage, b"Category\nFood\n", "bad.csv")
        .await
        .unwrap_err();
    assert!(error.to_string().contains("Nama Makanan"));
    assert_eq!(foods::search(&storage, "", 20).await.unwrap().len(), 0);
    cleanup(path).await;
}

#[tokio::test]
async fn imports_nama_schema_and_robust_numeric_formats() {
    let (storage, path) = storage().await;
    let csv = "nomor;kode;nama;kelompok;tipe;Energi (Energy);Protein (Protein);Lemak (Fat)\n1;X;Rice;Grain;Raw;1,147.1 Kal;147.1 g;0.33mg\n";
    assert_eq!(
        import::import_csv(&storage, csv.as_bytes(), "panganku.csv")
            .await
            .unwrap(),
        1
    );
    let result = foods::search(&storage, "rice", 20).await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].nutrients.get("energi"), Some(&1147.1));
    assert_eq!(result[0].nutrients.get("protein"), Some(&147.1));
    assert_eq!(result[0].nutrients.get("lemak total"), Some(&0.33));
    cleanup(path).await;
}

#[tokio::test]
async fn standard_import_upserts_duplicate_food_and_nutrients() {
    let (storage, path) = storage().await;
    let csv = "Nama Makanan;Kategori;Energi;Protein\nRice;Grain;100;2\nRice;Updated;200;3\n";
    assert_eq!(
        import::import_csv(&storage, csv.as_bytes(), "duplicate.csv")
            .await
            .unwrap(),
        2
    );
    assert_eq!(foods::search(&storage, "rice", 20).await.unwrap().len(), 1);
    let food = foods::search(&storage, "rice", 20)
        .await
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(food.category.as_deref(), Some("Updated"));
    assert_eq!(food.nutrients.get("energi"), Some(&200.0));
    assert_eq!(food.nutrients.get("protein"), Some(&3.0));
    cleanup(path).await;
}

#[tokio::test]
async fn failed_import_rolls_back_food_nutrients_and_readiness_state() {
    let (storage, path) = storage().await;
    let error = import::import_csv(
        &storage,
        b"Nama Makanan;Protein\nGood;2\n\"Broken;1\n",
        "rollback.csv",
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("rollback.csv"));
    assert!(foods::search(&storage, "", 20).await.unwrap().is_empty());
    assert!(!import::seed_is_complete(&storage).await.unwrap());
    cleanup(path).await;
}

#[tokio::test]
async fn food_status_is_not_ready_before_seed_completion() {
    let (storage, path) = storage().await;
    let status = foods::status(&storage).await.unwrap();
    assert_eq!(status.food_count, 0);
    assert!(!status.is_ready);
    cleanup(path).await;
}

#[tokio::test]
async fn food_status_is_ready_after_empty_seed_completion() {
    let (storage, path) = storage().await;
    import::seed_csvs(&storage, &[]).await.unwrap();
    let status = foods::status(&storage).await.unwrap();
    assert_eq!(status.food_count, 0);
    assert!(status.is_ready);
    cleanup(path).await;
}

#[tokio::test]
async fn food_status_stays_not_ready_for_manually_populated_database() {
    let (storage, path) = storage().await;
    sqlx::query("INSERT INTO foods (name, normalized_name) VALUES ('Manual food', 'manual food')")
        .execute(storage.pool())
        .await
        .unwrap();
    let status = foods::status(&storage).await.unwrap();
    assert_eq!(status.food_count, 1);
    assert!(!status.is_ready);
    cleanup(path).await;
}

#[tokio::test]
async fn seed_state_is_explicit_and_empty_resources_complete_deterministically() {
    let (storage, path) = storage().await;
    assert!(!import::seed_is_complete(&storage).await.unwrap());
    import::seed_csvs(&storage, &[]).await.unwrap();
    assert!(import::seed_is_complete(&storage).await.unwrap());
    cleanup(path).await;
}

#[tokio::test]
async fn bundled_seed_rolls_back_all_resources_and_completion_on_failure() {
    let (storage, path) = storage().await;
    let resources = [
        ("first.csv", b"Nama Makanan;Protein\nGood;2\n".as_slice()),
        ("broken.csv", b"\"unterminated\n".as_slice()),
    ];
    assert!(import::seed_csvs(&storage, &resources).await.is_err());
    assert!(foods::search(&storage, "", 20).await.unwrap().is_empty());
    assert!(!import::seed_is_complete(&storage).await.unwrap());
    cleanup(path).await;
}

#[tokio::test]
async fn canonical_aliases_preserve_frontend_nutrient_keys_and_unknowns() {
    let (storage, path) = storage().await;
    let csv = "Nama Makanan;Energi (Energy);Protein (Protein);Lemak (Fat);Karbohidrat (CHO);Serat Istimewa\nFood;100;2;3;4;5\n";
    import::import_csv(&storage, csv.as_bytes(), "aliases.csv")
        .await
        .unwrap();
    let food = foods::search(&storage, "food", 20)
        .await
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(food.nutrients.get("energi"), Some(&100.0));
    assert_eq!(food.nutrients.get("protein"), Some(&2.0));
    assert_eq!(food.nutrients.get("lemak total"), Some(&3.0));
    assert_eq!(food.nutrients.get("karbohidrat total"), Some(&4.0));
    assert_eq!(food.nutrients.get("serat istimewa"), Some(&5.0));
    cleanup(path).await;
}

#[tokio::test]
async fn bundled_panganku_header_and_row_imports_with_canonical_aliases() {
    let (storage, path) = storage().await;
    let bytes = include_bytes!("../../DatabaseMakanan/DataPangankuKemenkes2017.csv");
    assert_eq!(
        import::import_csv(&storage, bytes, "DataPangankuKemenkes2017.csv")
            .await
            .unwrap(),
        1146
    );
    let food = foods::search(&storage, "abon haruwan", 20)
        .await
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(food.nutrients.get("energi"), Some(&513.0));
    assert_eq!(food.nutrients.get("protein"), Some(&23.7));
    assert_eq!(food.nutrients.get("lemak total"), Some(&37.0));
    cleanup(path).await;
}

#[tokio::test]
async fn bundled_fatsecret_header_and_row_imports_with_canonical_aliases() {
    let (storage, path) = storage().await;
    let bytes = include_bytes!("../../DatabaseMakanan/DatabaseFatSecret.csv");
    assert_eq!(
        import::import_csv(&storage, bytes, "DatabaseFatSecret.csv")
            .await
            .unwrap(),
        4320
    );
    let food = foods::search(&storage, "0% fat yogurt", 20)
        .await
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(food.nutrients.get("energi"), Some(&51.0));
    assert_eq!(food.nutrients.get("protein"), Some(&4.0));
    assert_eq!(food.nutrients.get("lemak total"), Some(&0.1));
    cleanup(path).await;
}

#[tokio::test]
async fn failed_import_preserves_existing_same_basename_copy() {
    let (storage, path) = storage().await;
    let source = "selected.csv";
    let destination = storage.app_data_dir().join("imports").join(source);
    tokio::fs::create_dir_all(destination.parent().unwrap())
        .await
        .unwrap();
    tokio::fs::write(&destination, b"previous valid content")
        .await
        .unwrap();
    let result = import::copy_and_import(&storage, b"not a valid csv\n", source).await;
    assert!(result.is_err());
    assert_eq!(
        tokio::fs::read(destination).await.unwrap(),
        b"previous valid content"
    );
    cleanup(path).await;
}

#[tokio::test]
async fn import_copy_helper_persists_source_bytes_before_import() {
    let (storage, path) = storage().await;
    let source = "selected.csv";
    let bytes = b"Nama Makanan;Protein\nCopied;7\n";
    assert_eq!(
        import::copy_and_import(&storage, bytes, source)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        tokio::fs::read(storage.app_data_dir().join("imports").join(source))
            .await
            .unwrap(),
        bytes
    );
    cleanup(path).await;
}

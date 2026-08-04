pub mod error;
pub mod models;
pub mod storage;

pub mod commands {
    #[tauri::command]
    pub fn ping() -> &'static str {
        "pong"
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![commands::ping])
        .run(tauri::generate_context!())
        .expect("error while running NutriSurvey");
}

#[cfg(test)]
mod tests {
    #[test]
    fn ping_returns_pong() {
        assert_eq!(crate::commands::ping(), "pong");
    }

    #[test]
    fn dto_payload_uses_frontend_keys() {
        let item = crate::models::MappedMealItem {
            meal_type: "SARAPAN".into(),
            requested_keyword: "rice".into(),
            matched_food_id: 7,
            matched_food_name: "Rice".into(),
            suggested_grams: 100,
            reference_grams: 100.0,
            calories: 130.0,
            protein: 2.7,
            fat: 0.3,
            carbohydrate: 28.2,
            nutrients: std::collections::HashMap::new(),
            reasoning: "match".into(),
        };
        let value = serde_json::to_value(item).unwrap();
        assert_eq!(value["meal_type"], "SARAPAN");
        assert_eq!(value["requested_keyword"], "rice");
        assert_eq!(value["matched_food_id"], 7);
        assert_eq!(value["suggested_grams"], 100);
    }
}

#[cfg(test)]
mod storage_tests {
    #[tokio::test]
    async fn opens_and_initializes_under_supplied_app_data_path() {
        let path = std::env::temp_dir().join(format!(
            "nutrisurvey-test-{}-{}.db",
            std::process::id(),
            "open"
        ));
        let storage = crate::storage::Storage::open_path(&path).await.unwrap();
        storage.initialize().await.unwrap();
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('foods', 'nutrients', 'food_nutrients')")
            .fetch_one(storage.pool())
            .await
            .unwrap();
        assert_eq!(count.0, 3);
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn schema_initializes_twice_safely() {
        let path = std::env::temp_dir().join(format!(
            "nutrisurvey-test-{}-{}.db",
            std::process::id(),
            "twice"
        ));
        let storage = crate::storage::Storage::open_path(&path).await.unwrap();
        storage.initialize().await.unwrap();
        storage.initialize().await.unwrap();
        let _ = std::fs::remove_file(path);
    }
}

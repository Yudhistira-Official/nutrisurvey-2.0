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
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static TEMP_DATABASE_ID: AtomicU64 = AtomicU64::new(0);

    fn temp_database_path(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = TEMP_DATABASE_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir()
            .join(format!("nutrisurvey-test-{unique}-{id}"))
            .join(format!("{name}.db"))
    }

    async fn open_temp_database(name: &str) -> (crate::storage::Storage, std::path::PathBuf) {
        let path = temp_database_path(name);
        let storage = crate::storage::Storage::open_path(&path).await.unwrap();
        (storage, path)
    }

    #[tokio::test]
    async fn open_initializes_schema_under_nested_path() {
        let (storage, path) = open_temp_database("open").await;
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('foods', 'nutrients', 'food_nutrients')",
        )
        .fetch_one(storage.pool())
        .await
        .unwrap();
        assert_eq!(count.0, 3);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[tokio::test]
    async fn schema_initializes_twice_safely() {
        let (storage, path) = open_temp_database("twice").await;
        storage.initialize().await.unwrap();
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[tokio::test]
    async fn schema_enforces_foreign_keys_indexes_and_unique_constraints() {
        let (storage, path) = open_temp_database("constraints").await;
        let foreign_keys: (i64,) = sqlx::query_as("PRAGMA foreign_keys")
            .fetch_one(storage.pool())
            .await
            .unwrap();
        assert_eq!(foreign_keys.0, 1);

        let indexes: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name IN ('idx_foods_normalized_name', 'idx_food_nutrients_food_id', 'idx_food_nutrients_nutrient_id')",
        )
        .fetch_one(storage.pool())
        .await
        .unwrap();
        assert_eq!(indexes.0, 3);

        sqlx::query("INSERT INTO foods (name, normalized_name) VALUES ('Rice', 'rice')")
            .execute(storage.pool())
            .await
            .unwrap();
        sqlx::query("INSERT INTO nutrients (name, normalized_name, unit) VALUES ('Calories', 'calories', 'kcal')")
            .execute(storage.pool())
            .await
            .unwrap();
        sqlx::query("INSERT INTO food_nutrients (food_id, nutrient_id, amount) VALUES (1, 1, 130)")
            .execute(storage.pool())
            .await
            .unwrap();
        assert!(sqlx::query("INSERT INTO food_nutrients (food_id, nutrient_id, amount) VALUES (1, 1, 130)")
            .execute(storage.pool())
            .await
            .is_err());
        assert!(sqlx::query("INSERT INTO food_nutrients (food_id, nutrient_id, amount) VALUES (99, 1, 130)")
            .execute(storage.pool())
            .await
            .is_err());
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}

pub mod ai;
pub mod error;
pub mod export;
pub mod foods;
pub mod import;
pub mod meals;
pub mod models;
pub mod nutrition;
pub mod storage;

use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

pub struct AppState {
    pub storage: Arc<storage::Storage>,
}

pub mod commands {
    use super::*;

    #[tauri::command]
    pub fn ping() -> &'static str {
        "pong"
    }

    #[tauri::command]
    pub async fn food_search(
        state: State<'_, AppState>,
        query: String,
        limit: Option<u32>,
    ) -> Result<Vec<models::FoodResult>, error::AppError> {
        foods::search(&state.storage, &query, limit.unwrap_or(20)).await
    }

    #[tauri::command]
    pub async fn food_status(
        state: State<'_, AppState>,
    ) -> Result<foods::FoodStatus, error::AppError> {
        foods::status(&state.storage).await
    }

    #[tauri::command]
    pub fn calculate_tdee(
        request: models::TdeeRequest,
    ) -> Result<models::TdeeResponse, error::AppError> {
        nutrition::calculate_tdee(request)
    }

    #[tauri::command]
    pub async fn food_recommendations(
        state: State<'_, AppState>,
        filters: Vec<models::RecommendationFilter>,
    ) -> Result<Vec<models::FoodResult>, error::AppError> {
        foods::recommend(&state.storage, &filters).await
    }

    #[tauri::command]
    pub async fn nutrient_list(
        state: State<'_, AppState>,
    ) -> Result<Vec<models::NutrientSummary>, error::AppError> {
        foods::list_nutrients(&state.storage).await
    }

    #[tauri::command]
    pub async fn import_food_csv(
        state: State<'_, AppState>,
        bytes: Vec<u8>,
        source_name: String,
    ) -> Result<u64, error::AppError> {
        import::copy_and_import(&state.storage, &bytes, &source_name).await
    }

    #[tauri::command]
    pub async fn generate_ai_menu(
        state: State<'_, AppState>,
        request: models::AiRequest,
    ) -> Result<Vec<models::MappedMealItem>, error::AppError> {
        ai::generate_menu(&state.storage, request).await
    }

    #[tauri::command]
    pub fn export_word(
        app: AppHandle,
        request: export::ExportRequest,
    ) -> Result<export::ExportResult, error::AppError> {
        let template = include_bytes!("../../Assets/template.rtf");
        let bytes = export::render_rtf(request, template)?;
        let initial = export::result(bytes.clone());

        #[cfg(desktop)]
        {
            use tauri_plugin_dialog::DialogExt;

            let selected = app
                .dialog()
                .file()
                .set_file_name(&initial.filename)
                .add_filter("Rich Text Format", &["rtf"])
                .blocking_save_file();
            let selected = export::resolve_selected_path(selected.map(|path| path.into_path()))?;
            let path = export::validate_selected_path(selected.as_deref())?;
            std::fs::write(path, &bytes)?;
            Ok(export::ExportResult {
                delivery: export::ExportDelivery::Saved,
                saved_path: Some(path.to_string_lossy().into_owned()),
                ..initial
            })
        }

        #[cfg(mobile)]
        {
            let _ = app;
            let _ = initial;
            export::mobile_delivery_error()
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let storage = tauri::async_runtime::block_on(storage::Storage::open(app.handle()))?;
            let storage = Arc::new(storage);
            tauri::async_runtime::block_on(seed_resources(&storage))?;
            app.manage(AppState { storage });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::food_search,
            commands::food_status,
            commands::calculate_tdee,
            commands::food_recommendations,
            commands::nutrient_list,
            commands::import_food_csv,
            commands::generate_ai_menu,
            commands::export_word
        ])
        .run(tauri::generate_context!())
        .expect("error while running NutriSurvey");
}

async fn seed_resources(storage: &storage::Storage) -> Result<(), error::AppError> {
    if import::seed_is_complete(storage).await? {
        return Ok(());
    }
    let mut resources = Vec::new();
    if storage.resource_dir().is_dir() {
        let mut entries = tokio::fs::read_dir(storage.resource_dir()).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) == Some("csv") {
                resources.push((
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("resource.csv")
                        .to_string(),
                    tokio::fs::read(path).await?,
                ));
            }
        }
    }
    let references = resources
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
        .collect::<Vec<_>>();
    import::seed_csvs(storage, &references).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn ping_returns_pong() {
        assert_eq!(crate::commands::ping(), "pong");
    }

    #[test]
    fn database_rows_map_normalized_names_without_api_leakage() {
        let food = crate::models::Food::from(crate::models::FoodRow {
            id: 1,
            name: "Rice".into(),
            _normalized_name: "rice".into(),
            brand: None,
            category: None,
            serving_size: 100.0,
            serving_unit: "g".into(),
            servings_per_container: 1.0,
        });
        let nutrient = crate::models::Nutrient::from(crate::models::NutrientRow {
            id: 2,
            name: "Calories".into(),
            _normalized_name: "calories".into(),
            unit: "kcal".into(),
        });

        assert_eq!(food.name, "Rice");
        assert_eq!(nutrient.name, "Calories");
        assert!(!serde_json::to_value(food)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("normalizedName"));
        assert!(!serde_json::to_value(nutrient)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("normalizedName"));
    }

    #[test]
    fn all_task_two_dtos_use_compatible_json_keys() {
        let values = [
            serde_json::to_value(crate::models::Food {
                id: 1,
                name: "Rice".into(),
                brand: None,
                category: None,
                serving_size: 100.0,
                serving_unit: "g".into(),
                servings_per_container: 1.0,
            })
            .unwrap(),
            serde_json::to_value(crate::models::Nutrient {
                id: 2,
                name: "Calories".into(),
                unit: "kcal".into(),
            })
            .unwrap(),
            serde_json::to_value(crate::models::FoodNutrient {
                id: 3,
                food_id: 1,
                nutrient_id: 2,
                amount: 130.0,
            })
            .unwrap(),
            serde_json::to_value(crate::models::FoodResult {
                id: 1,
                name: "Rice".into(),
                brand: None,
                category: None,
                serving_size: 100.0,
                serving_unit: "g".into(),
                servings_per_container: 1.0,
                nutrients: std::collections::HashMap::from([("Calories".into(), 130.0)]),
            })
            .unwrap(),
            serde_json::to_value(crate::models::NutrientSummary {
                name: "Calories".into(),
                unit: "kcal".into(),
                amount: 130.0,
            })
            .unwrap(),
            serde_json::to_value(crate::models::TdeeRequest {
                weight_kg: 70.0,
                height_cm: 175.0,
                age: 30,
                gender: "male".into(),
                activity_factor: 1.2,
                injury_factor: 1.0,
                is_manual_factors: false,
            })
            .unwrap(),
            serde_json::to_value(crate::models::TdeeResponse {
                basal_metabolic_rate: 1600.0,
                total_daily_energy_expenditure: 1920.0,
                formula_used: "Mifflin".into(),
            })
            .unwrap(),
            serde_json::to_value(crate::models::RecommendationFilter {
                nutrient: "Protein".into(),
                operator: ">=".into(),
                value: 20.0,
            })
            .unwrap(),
            serde_json::to_value(crate::models::AiConfig {
                provider: "openai".into(),
                model: "gpt".into(),
                api_key: "secret".into(),
                base_url: "https://example.test".into(),
            })
            .unwrap(),
            serde_json::to_value(crate::models::MappedMealItem {
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
            })
            .unwrap(),
        ];
        let expected_keys = [
            vec![
                "id",
                "name",
                "brand",
                "category",
                "servingSize",
                "servingUnit",
                "servingsPerContainer",
            ],
            vec!["id", "name", "unit"],
            vec!["id", "foodId", "nutrientId", "amount"],
            vec![
                "id",
                "name",
                "brand",
                "category",
                "servingSize",
                "servingUnit",
                "servingsPerContainer",
                "nutrients",
            ],
            vec!["name", "unit", "amount"],
            vec![
                "weightKg",
                "heightCm",
                "age",
                "gender",
                "activityFactor",
                "injuryFactor",
                "isManualFactors",
            ],
            vec![
                "basalMetabolicRate",
                "totalDailyEnergyExpenditure",
                "formulaUsed",
            ],
            vec!["nutrient", "operator", "value"],
            vec!["provider", "model", "apiKey", "baseUrl"],
            vec![
                "meal_type",
                "requested_keyword",
                "matched_food_id",
                "matched_food_name",
                "suggested_grams",
                "reference_grams",
                "calories",
                "protein",
                "fat",
                "carbohydrate",
                "nutrients",
                "reasoning",
            ],
        ];
        for (value, keys) in values.into_iter().zip(expected_keys) {
            let actual = value
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<std::collections::BTreeSet<_>>();
            let expected = keys.into_iter().collect::<std::collections::BTreeSet<_>>();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn app_errors_use_compatible_json_shape() {
        for error in [
            crate::error::AppError::Validation("bad input".into()),
            crate::error::AppError::Database("db failed".into()),
            crate::error::AppError::Import("import failed".into()),
            crate::error::AppError::Ai("ai failed".into()),
            crate::error::AppError::Export("export failed".into()),
            crate::error::AppError::Io("io failed".into()),
        ] {
            let value = serde_json::to_value(error).unwrap();
            assert_eq!(value.as_object().unwrap().len(), 2);
            assert!(value.get("kind").unwrap().is_string());
            assert!(value.get("message").unwrap().is_string());
        }
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
        assert!(sqlx::query("INSERT INTO nutrients (name, normalized_name, unit) VALUES ('Energy', 'calories', 'kcal')")
            .execute(storage.pool())
            .await
            .is_err());
        sqlx::query("INSERT INTO food_nutrients (food_id, nutrient_id, amount) VALUES (1, 1, 130)")
            .execute(storage.pool())
            .await
            .unwrap();
        assert!(sqlx::query(
            "INSERT INTO food_nutrients (food_id, nutrient_id, amount) VALUES (1, 1, 130)"
        )
        .execute(storage.pool())
        .await
        .is_err());
        assert!(sqlx::query(
            "INSERT INTO food_nutrients (food_id, nutrient_id, amount) VALUES (99, 1, 130)"
        )
        .execute(storage.pool())
        .await
        .is_err());
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}

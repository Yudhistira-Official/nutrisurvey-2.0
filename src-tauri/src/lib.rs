pub mod ai;
pub mod error;
pub mod export;
pub mod foods;
pub mod import;
pub mod meals;
pub mod models;
pub mod nutrition;
pub mod project;
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
    pub async fn check_update(app: AppHandle) -> Result<Option<String>, error::AppError> {
        use tauri_plugin_updater::UpdaterExt;
        let update = app
            .updater()
            .map_err(|cause| error::AppError::Io(cause.to_string()))?
            .check()
            .await
            .map_err(|cause| error::AppError::Io(cause.to_string()))?;
        Ok(update.map(|update| update.version.to_string()))
    }

    #[tauri::command]
    pub async fn install_update(app: AppHandle) -> Result<(), error::AppError> {
        use tauri_plugin_updater::UpdaterExt;
        let Some(update) = app
            .updater()
            .map_err(|cause| error::AppError::Io(cause.to_string()))?
            .check()
            .await
            .map_err(|cause| error::AppError::Io(cause.to_string()))?
        else {
            return Ok(());
        };
        update
            .download_and_install(|_bytes, _total| {}, || {})
            .await
            .map_err(|cause| error::AppError::Io(cause.to_string()))?;
        Ok(())
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
    pub async fn import_csv_from_path(
        state: State<'_, AppState>,
        path: String,
    ) -> Result<u64, error::AppError> {
        let bytes = std::fs::read(&path).map_err(|error| error::AppError::Io(error.to_string()))?;
        let source_name = std::path::Path::new(&path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("import.csv")
            .to_owned();
        import::copy_and_import(&state.storage, &bytes, &source_name).await
    }

    #[tauri::command]
    pub fn ai_key_load() -> Result<Option<String>, error::AppError> {
        let entry = keyring::Entry::new("NutriSurvey", "ai-api-key")
            .map_err(|cause| error::AppError::Io(cause.to_string()))?;
        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(cause) => Err(error::AppError::Io(cause.to_string())),
        }
    }

    #[tauri::command]
    pub fn ai_key_save(value: String) -> Result<(), error::AppError> {
        let entry = keyring::Entry::new("NutriSurvey", "ai-api-key")
            .map_err(|cause| error::AppError::Io(cause.to_string()))?;
        entry
            .set_password(&value)
            .map_err(|cause| error::AppError::Io(cause.to_string()))
    }

    #[tauri::command]
    pub fn ai_key_delete() -> Result<(), error::AppError> {
        let entry = keyring::Entry::new("NutriSurvey", "ai-api-key")
            .map_err(|cause| error::AppError::Io(cause.to_string()))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(cause) => Err(error::AppError::Io(cause.to_string())),
        }
    }

    #[tauri::command]
    pub fn ai_default_info() -> models::AiDefaultInfo {
        models::AiDefaultInfo {
            available: option_env!("AI_API_KEY").is_some() && option_env!("AI_MODEL").is_some(),
            provider: option_env!("AI_PROVIDER").unwrap_or("openrouter").into(),
            model: option_env!("AI_MODEL").unwrap_or("").into(),
            base_url: option_env!("AI_BASE_URL")
                .or(option_env!("BASE_URL"))
                .unwrap_or("")
                .into(),
        }
    }

    #[tauri::command]
    pub async fn generate_ai_menu(
        state: State<'_, AppState>,
        mut request: models::AiRequest,
    ) -> Result<Vec<models::MappedMealItem>, error::AppError> {
        if request.provider == "builtin_default" {
            request.provider = option_env!("AI_PROVIDER").unwrap_or("openrouter").into();
            request.model = option_env!("AI_MODEL").unwrap_or("").into();
            request.base_url = option_env!("AI_BASE_URL")
                .or(option_env!("BASE_URL"))
                .unwrap_or("")
                .into();
            request.api_key = option_env!("AI_API_KEY").unwrap_or("").into();
        }
        ai::generate_menu(&state.storage, request).await
    }

    #[tauri::command]
    pub fn cancel_ai_request(request_id: String) {
        ai::cancel_request(&request_id);
    }

    #[tauri::command]
    pub async fn stream_ai_menu(
        state: State<'_, AppState>,
        mut request: models::AiRequest,
        channel: tauri::ipc::Channel<String>,
    ) -> Result<Vec<models::MappedMealItem>, error::AppError> {
        if request.provider == "builtin_default" {
            request.provider = option_env!("AI_PROVIDER").unwrap_or("openrouter").into();
            request.model = option_env!("AI_MODEL").unwrap_or("").into();
            request.base_url = option_env!("AI_BASE_URL")
                .or(option_env!("BASE_URL"))
                .unwrap_or("")
                .into();
            request.api_key = option_env!("AI_API_KEY").unwrap_or("").into();
        }
        ai::stream_menu(&state.storage, request, channel).await
    }

    #[tauri::command]
    pub async fn file_history_list(
        state: State<'_, AppState>,
    ) -> Result<Vec<models::FileHistoryItem>, error::AppError> {
        let raw = sqlx::query_scalar::<_, String>(
            "SELECT value FROM app_state WHERE key = 'file_history'",
        )
        .fetch_optional(state.storage.pool())
        .await?;
        let items = raw
            .map(|value| {
                serde_json::from_str::<Vec<models::FileHistoryItem>>(&value).unwrap_or_default()
            })
            .unwrap_or_default();
        Ok(items
            .into_iter()
            .filter(|item| std::path::Path::new(&item.path).is_file())
            .collect())
    }

    async fn record_history(
        storage: &storage::Storage,
        item: models::FileHistoryItem,
    ) -> Result<(), error::AppError> {
        if !std::path::Path::new(&item.path).is_file() {
            return Err(error::AppError::Validation("file does not exist".into()));
        }
        let raw = sqlx::query_scalar::<_, String>(
            "SELECT value FROM app_state WHERE key = 'file_history'",
        )
        .fetch_optional(storage.pool())
        .await?;
        let mut items = raw
            .map(|value| {
                serde_json::from_str::<Vec<models::FileHistoryItem>>(&value).unwrap_or_default()
            })
            .unwrap_or_default();
        items.retain(|entry| entry.path != item.path || entry.kind != item.kind);
        items.insert(0, item);
        items.truncate(50);
        let value = serde_json::to_string(&items)
            .map_err(|cause| error::AppError::Io(cause.to_string()))?;
        sqlx::query("INSERT INTO app_state (key,value) VALUES ('file_history',?) ON CONFLICT(key) DO UPDATE SET value=excluded.value").bind(value).execute(storage.pool()).await?;
        Ok(())
    }

    #[tauri::command]
    pub async fn file_history_record(
        state: State<'_, AppState>,
        item: models::FileHistoryItem,
    ) -> Result<(), error::AppError> {
        if !std::path::Path::new(&item.path).is_file() {
            return Err(error::AppError::Validation("file does not exist".into()));
        }
        record_history(&state.storage, item).await
    }

    #[tauri::command]
    pub fn open_existing_file(path: String) -> Result<(), error::AppError> {
        let path_ref = std::path::Path::new(&path);
        if !path_ref.is_file() {
            return Err(error::AppError::Validation("file does not exist".into()));
        }
        #[cfg(target_os = "linux")]
        let mut command = std::process::Command::new("xdg-open");
        #[cfg(target_os = "macos")]
        let mut command = std::process::Command::new("open");
        #[cfg(target_os = "windows")]
        let mut command = {
            let mut command = std::process::Command::new("cmd");
            command.args(["/C", "start", ""]);
            command
        };
        command
            .arg(path_ref)
            .spawn()
            .map_err(|cause| error::AppError::Io(cause.to_string()))?;
        Ok(())
    }

    #[tauri::command]
    pub async fn project_save(
        app: AppHandle,
        project: project::ProjectFile,
    ) -> Result<bool, error::AppError> {
        #[cfg(desktop)]
        {
            use tauri_plugin_dialog::DialogExt;
            let (tx, rx) = tokio::sync::oneshot::channel();
            app.dialog()
                .file()
                .set_file_name("Nutri.nutri")
                .add_filter("Nutri project", &["nutri"])
                .save_file(move |selected| {
                    let _ = tx.send(selected);
                });
            let selected = rx
                .await
                .map_err(|_| error::AppError::Io("dialog task dropped".into()))?;
            let Some(path) = export::resolve_selected_path(selected.map(|path| path.into_path()))?
            else {
                return Ok(false);
            };
            project::save(&path, &project).await?;
            record_history(
                &app.state::<AppState>().storage,
                models::FileHistoryItem {
                    path: path.to_string_lossy().into_owned(),
                    kind: "project".into(),
                },
            )
            .await?;
            Ok(true)
        }
        #[cfg(mobile)]
        {
            let _ = (app, project);
            Err(error::AppError::Io("Proyek mobile belum didukung".into()))
        }
    }

    #[tauri::command]
    pub async fn project_open_path(
        state: State<'_, AppState>,
        path: String,
    ) -> Result<project::ProjectFile, error::AppError> {
        let path = std::path::PathBuf::from(path);
        let project = project::load(&path).await?;
        record_history(
            &state.storage,
            models::FileHistoryItem {
                path: path.to_string_lossy().into_owned(),
                kind: "project".into(),
            },
        )
        .await?;
        Ok(project)
    }

    #[tauri::command]
    pub async fn project_open(
        app: AppHandle,
    ) -> Result<Option<project::ProjectFile>, error::AppError> {
        #[cfg(desktop)]
        {
            use tauri_plugin_dialog::DialogExt;
            let (tx, rx) = tokio::sync::oneshot::channel();
            app.dialog()
                .file()
                .add_filter("Nutri project", &["nutri"])
                .pick_file(move |selected| {
                    let _ = tx.send(selected);
                });
            let selected = rx
                .await
                .map_err(|_| error::AppError::Io("dialog task dropped".into()))?;
            let Some(path) = export::resolve_selected_path(selected.map(|path| path.into_path()))?
            else {
                return Ok(None);
            };
            let project = project::load(&path).await?;
            record_history(
                &app.state::<AppState>().storage,
                models::FileHistoryItem {
                    path: path.to_string_lossy().into_owned(),
                    kind: "project".into(),
                },
            )
            .await?;
            Ok(Some(project))
        }
        #[cfg(mobile)]
        {
            let _ = app;
            Err(error::AppError::Io("Proyek mobile belum didukung".into()))
        }
    }

    pub trait ExportDeliveryHandler {
        fn deliver(
            self,
            result: export::ExportResult,
        ) -> Result<export::ExportResult, error::AppError>;
    }

    impl<F> ExportDeliveryHandler for F
    where
        F: FnOnce(export::ExportResult) -> Result<export::ExportResult, error::AppError>,
    {
        fn deliver(
            self,
            result: export::ExportResult,
        ) -> Result<export::ExportResult, error::AppError> {
            self(result)
        }
    }

    pub fn export_word_with_handler<H>(
        request: export::ExportRequest,
        handler: H,
    ) -> Result<export::ExportResult, error::AppError>
    where
        H: ExportDeliveryHandler,
    {
        let template = include_bytes!("../../Assets/template.rtf");
        let bytes = export::render_rtf(request, template)?;
        handler.deliver(export::result(bytes))
    }

    #[tauri::command]
    pub async fn export_word(
        app: AppHandle,
        request: export::ExportRequest,
    ) -> Result<export::ExportResult, error::AppError> {
        #[cfg(desktop)]
        {
            use tauri_plugin_dialog::DialogExt;

            let initial = export_word_with_handler(request, Ok)?;
            let (tx, rx) = tokio::sync::oneshot::channel();
            let filename = initial.filename.clone();
            app.dialog()
                .file()
                .set_file_name(&filename)
                .add_filter("Rich Text Format", &["rtf"])
                .save_file(move |selected| {
                    let _ = tx.send(selected);
                });
            let selected = rx
                .await
                .map_err(|_| error::AppError::Io("dialog task dropped".into()))?;
            let selected = export::resolve_selected_path(selected.map(|path| path.into_path()))?;
            let path = export::validate_selected_path(selected.as_deref())?;
            std::fs::write(path, &initial.bytes)?;
            record_history(
                &app.state::<AppState>().storage,
                models::FileHistoryItem {
                    path: path.to_string_lossy().into_owned(),
                    kind: "report".into(),
                },
            )
            .await?;
            Ok(export::ExportResult {
                delivery: export::ExportDelivery::Saved,
                saved_path: Some(path.to_string_lossy().into_owned()),
                ..initial
            })
        }

        #[cfg(mobile)]
        {
            let _ = app;
            export_word_with_handler(request, |_initial: export::ExportResult| {
                export::mobile_delivery_error()
            })
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let storage = tauri::async_runtime::block_on(storage::Storage::open(app.handle()))?;
            let storage = Arc::new(storage);
            tauri::async_runtime::block_on(seed_configured_resources(&storage))?;
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
            commands::import_csv_from_path,
            commands::generate_ai_menu,
            commands::stream_ai_menu,
            commands::cancel_ai_request,
            commands::ai_default_info,
            commands::ai_key_load,
            commands::ai_key_save,
            commands::ai_key_delete,
            commands::file_history_list,
            commands::file_history_record,
            commands::open_existing_file,
            commands::project_save,
            commands::project_open,
            commands::project_open_path,
            commands::export_word,
            commands::check_update,
            commands::install_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running NutriSurvey");
}

pub async fn seed_configured_resources(storage: &storage::Storage) -> Result<u64, error::AppError> {
    let resource_dir = storage.resource_dir();
    let mut paths = Vec::new();
    for dir in [
        Some(resource_dir.to_path_buf()),
        resource_dir.parent().map(|p| p.join("DatabaseMakanan")),
        Some(resource_dir.join("DatabaseMakanan")),
    ]
    .iter()
    .flatten()
    {
        if dir.is_dir() {
            let mut entries = tokio::fs::read_dir(dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.extension().and_then(|extension| extension.to_str()) == Some("csv") {
                    paths.push(path);
                }
            }
        }
    }
    if paths.is_empty() {
        let project_db = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("DatabaseMakanan");
        if project_db.is_dir() {
            let mut entries = tokio::fs::read_dir(project_db).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.extension().and_then(|extension| extension.to_str()) == Some("csv") {
                    paths.push(path);
                }
            }
        }
    }
    paths.sort();
    paths.dedup();
    let mut resources = Vec::new();
    for path in paths {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("resource.csv")
            .to_string();
        resources.push((name, tokio::fs::read(path).await?));
    }
    let references = resources
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
        .collect::<Vec<_>>();
    import::synchronize_sources(storage, &references).await
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
                bmi_standard: "asia_pacific".into(),
            })
            .unwrap(),
            serde_json::to_value(crate::models::TdeeResponse {
                basal_metabolic_rate: 1600.0,
                total_daily_energy_expenditure: 1920.0,
                formula_used: "Mifflin".into(),
                bmi: 22.86,
                nutrition_classification: "Normal".into(),
                ideal_weight: 67.5,
                adjusted_weight: 68.13,
                reference_weight: 70.0,
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
                "bmiStandard",
            ],
            vec![
                "basalMetabolicRate",
                "totalDailyEnergyExpenditure",
                "formulaUsed",
                "bmi",
                "nutritionClassification",
                "idealWeight",
                "adjustedWeight",
                "referenceWeight",
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
    fn ai_request_accepts_frontend_camel_case_payload() {
        let value = serde_json::json!({
            "targetTdee": 2000,
            "targetCarbs": 250,
            "targetProtein": 100,
            "targetFat": 60,
            "availableMealTypes": ["Makan Pagi"],
            "provider": "openrouter",
            "model": "openai/gpt-4o-mini",
            "apiKey": "test-key",
            "baseUrl": "https://openrouter.ai/api/v1"
        });
        let request: crate::models::AiRequest = serde_json::from_value(value).unwrap();
        assert_eq!(request.target_tdee, 2000);
        assert_eq!(request.available_meal_types, vec!["Makan Pagi"]);
        assert_eq!(request.api_key, "test-key");
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

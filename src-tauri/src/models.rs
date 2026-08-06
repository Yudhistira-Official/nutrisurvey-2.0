use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Food {
    pub id: i64,
    pub name: String,
    pub brand: Option<String>,
    pub category: Option<String>,
    pub serving_size: f64,
    pub serving_unit: String,
    pub servings_per_container: f64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub(crate) struct FoodRow {
    pub id: i64,
    pub name: String,
    #[sqlx(rename = "normalized_name")]
    pub _normalized_name: String,
    pub brand: Option<String>,
    pub category: Option<String>,
    pub serving_size: f64,
    pub serving_unit: String,
    pub servings_per_container: f64,
}

impl From<FoodRow> for Food {
    fn from(row: FoodRow) -> Self {
        let FoodRow {
            id,
            name,
            _normalized_name: _,
            brand,
            category,
            serving_size,
            serving_unit,
            servings_per_container,
        } = row;
        Self {
            id,
            name,
            brand,
            category,
            serving_size,
            serving_unit,
            servings_per_container,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Nutrient {
    pub id: i64,
    pub name: String,
    pub unit: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub(crate) struct NutrientRow {
    pub id: i64,
    pub name: String,
    #[sqlx(rename = "normalized_name")]
    pub _normalized_name: String,
    pub unit: String,
}

impl From<NutrientRow> for Nutrient {
    fn from(row: NutrientRow) -> Self {
        let NutrientRow {
            id,
            name,
            _normalized_name: _,
            unit,
        } = row;
        Self { id, name, unit }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FoodNutrient {
    pub id: i64,
    pub food_id: i64,
    pub nutrient_id: i64,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FoodResult {
    pub id: i64,
    pub name: String,
    pub brand: Option<String>,
    pub category: Option<String>,
    pub serving_size: f64,
    pub serving_unit: String,
    pub servings_per_container: f64,
    pub nutrients: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct NutrientSummary {
    pub name: String,
    pub unit: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TdeeRequest {
    pub weight_kg: f64,
    pub height_cm: f64,
    pub age: i32,
    pub gender: String,
    pub activity_factor: f64,
    pub injury_factor: f64,
    pub is_manual_factors: bool,
    pub bmi_standard: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TdeeResponse {
    pub basal_metabolic_rate: f64,
    pub total_daily_energy_expenditure: f64,
    pub formula_used: String,
    pub bmi: f64,
    pub nutrition_classification: String,
    pub ideal_weight: f64,
    pub adjusted_weight: f64,
    pub reference_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHistoryItem {
    pub path: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationFilter {
    pub nutrient: String,
    pub operator: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiDefaultInfo {
    pub available: bool,
    pub provider: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRequest {
    pub target_tdee: i32,
    pub target_carbs: i32,
    pub target_protein: i32,
    pub target_fat: i32,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub active_menu: String,
    #[serde(default)]
    pub revision: bool,
    #[serde(default)]
    pub verify_menu: bool,
    #[serde(default)]
    pub candidate_catalog: String,
    #[serde(default)]
    pub request_id: String,
    pub available_meal_types: Vec<String>,
    pub provider: String,
    pub model: String,
    #[serde(skip_serializing)]
    pub api_key: String,
    pub base_url: String,
}

impl std::fmt::Debug for AiRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AiRequest")
            .field("target_tdee", &self.target_tdee)
            .field("target_carbs", &self.target_carbs)
            .field("target_protein", &self.target_protein)
            .field("target_fat", &self.target_fat)
            .field("available_meal_types", &self.available_meal_types)
            .field("provider", &self.provider)
            .field("model", &self.model)
            .field("api_key", &"[REDACTED]")
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMealInput {
    pub meal_type: String,
    pub food_keyword: String,
    pub suggested_grams: i32,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappedMealItem {
    #[serde(rename = "meal_type")]
    pub meal_type: String,
    #[serde(rename = "requested_keyword")]
    pub requested_keyword: String,
    #[serde(rename = "matched_food_id")]
    pub matched_food_id: i64,
    #[serde(rename = "matched_food_name")]
    pub matched_food_name: String,
    #[serde(rename = "suggested_grams")]
    pub suggested_grams: i32,
    #[serde(rename = "reference_grams")]
    pub reference_grams: f64,
    pub calories: f64,
    pub protein: f64,
    pub fat: f64,
    pub carbohydrate: f64,
    pub nutrients: HashMap<String, f64>,
    pub reasoning: String,
}

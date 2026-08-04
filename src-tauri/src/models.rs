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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TdeeResponse {
    pub basal_metabolic_rate: f64,
    pub total_daily_energy_expenditure: f64,
    pub formula_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationFilter {
    pub nutrient: String,
    pub operator: String,
    pub value: f64,
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

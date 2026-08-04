use crate::{
    error::AppError,
    import::canonical_nutrient_name,
    models::{FoodResult, NutrientSummary},
    storage::Storage,
};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
struct FoodSearchRow {
    id: i64,
    name: String,
    brand: Option<String>,
    category: Option<String>,
    serving_size: f64,
    serving_unit: String,
    servings_per_container: f64,
    nutrient_name: Option<String>,
    amount: Option<f64>,
}

pub async fn search(
    storage: &Storage,
    query: &str,
    limit: u32,
) -> Result<Vec<FoodResult>, AppError> {
    let cap = if query.trim().is_empty() {
        5
    } else {
        limit.min(20)
    };
    let rows = sqlx::query_as::<_, FoodSearchRow>(
        "SELECT f.id, f.name, f.brand, f.category, f.serving_size, f.serving_unit, f.servings_per_container, n.name AS nutrient_name, fn.amount FROM (SELECT * FROM foods WHERE (? = '' OR normalized_name LIKE '%' || ? || '%') ORDER BY name LIMIT ?) f LEFT JOIN food_nutrients fn ON fn.food_id = f.id LEFT JOIN nutrients n ON n.id = fn.nutrient_id ORDER BY f.name",
    )
    .bind(query.trim().to_lowercase()).bind(query.trim().to_lowercase()).bind(cap as i64)
    .fetch_all(storage.pool()).await?;
    let mut result = Vec::new();
    for row in rows {
        if let Some(food) = result
            .iter_mut()
            .find(|food: &&mut FoodResult| food.id == row.id)
        {
            if let (Some(name), Some(amount)) = (row.nutrient_name, row.amount) {
                food.nutrients
                    .insert(canonical_nutrient_name(&name), amount);
            }
        } else {
            let mut nutrients = std::collections::HashMap::new();
            if let (Some(name), Some(amount)) = (row.nutrient_name, row.amount) {
                nutrients.insert(canonical_nutrient_name(&name), amount);
            }
            result.push(FoodResult {
                id: row.id,
                name: row.name,
                brand: row.brand,
                category: row.category,
                serving_size: row.serving_size,
                serving_unit: row.serving_unit,
                servings_per_container: row.servings_per_container,
                nutrients,
            });
        }
    }
    Ok(result)
}

pub async fn status(storage: &Storage) -> Result<FoodStatus, AppError> {
    let food_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM foods")
        .fetch_one(storage.pool())
        .await?;
    Ok(FoodStatus {
        food_count,
        is_ready: food_count > 0,
    })
}

pub async fn list_nutrients(storage: &Storage) -> Result<Vec<NutrientSummary>, AppError> {
    let rows =
        sqlx::query_as::<_, (String, String)>("SELECT name, unit FROM nutrients ORDER BY name")
            .fetch_all(storage.pool())
            .await?;
    let mut result = Vec::new();
    for (name, unit) in rows {
        let canonical = canonical_nutrient_name(&name);
        if !result
            .iter()
            .any(|item: &NutrientSummary| item.name == canonical)
        {
            result.push(NutrientSummary {
                name: canonical,
                unit,
                amount: 0.0,
            });
        }
    }
    Ok(result)
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoodStatus {
    pub food_count: i64,
    pub is_ready: bool,
}

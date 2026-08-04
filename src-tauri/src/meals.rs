use crate::{
    error::AppError,
    import::canonical_nutrient_name,
    models::{AiMealInput, FoodResult, MappedMealItem},
    storage::Storage,
};

pub async fn map_ai_items(
    storage: &Storage,
    items: &[AiMealInput],
) -> Result<Vec<MappedMealItem>, AppError> {
    let mut mapped = Vec::new();
    for item in items {
        let keyword = normalize(&item.food_keyword);
        if keyword.is_empty() || item.suggested_grams <= 0 {
            continue;
        }
        let candidates = crate::foods::candidate_foods(storage, &keyword, 25).await?;
        let food = candidates.into_iter().min_by_key(|food| {
            (
                match_score(&normalize(&food.name), &keyword),
                food.name.len(),
            )
        });
        if let Some(food) = food {
            mapped.push(map_food(item, food));
        }
    }
    Ok(mapped)
}

fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn match_score(name: &str, keyword: &str) -> u8 {
    if name == keyword {
        0
    } else if name.starts_with(keyword) {
        1
    } else if name.contains(keyword) {
        2
    } else {
        3
    }
}

fn map_food(item: &AiMealInput, food: FoodResult) -> MappedMealItem {
    let reference_grams = if food.serving_size > 0.0 {
        food.serving_size
    } else {
        100.0
    };
    let multiplier = item.suggested_grams as f64 / reference_grams;
    let nutrients = food
        .nutrients
        .into_iter()
        .map(|(name, amount)| (name, round(amount * multiplier)))
        .collect::<std::collections::HashMap<_, _>>();
    MappedMealItem {
        meal_type: item.meal_type.clone(),
        requested_keyword: item.food_keyword.clone(),
        matched_food_id: food.id,
        matched_food_name: food.name,
        suggested_grams: item.suggested_grams,
        reference_grams,
        calories: nutrient_value(&nutrients, &["energi", "energy", "calories", "kcal"]),
        protein: nutrient_value(&nutrients, &["protein"]),
        fat: nutrient_value(&nutrients, &["lemak total", "lemak", "fat", "total fat"]),
        carbohydrate: nutrient_value(
            &nutrients,
            &[
                "karbohidrat total",
                "karbohidrat",
                "carbohydrate",
                "carbs",
                "total carbohydrate",
            ],
        ),
        nutrients,
        reasoning: item.reasoning.clone(),
    }
}

fn nutrient_value(nutrients: &std::collections::HashMap<String, f64>, aliases: &[&str]) -> f64 {
    aliases
        .iter()
        .find_map(|alias| {
            nutrients
                .get(&canonical_nutrient_name(alias))
                .copied()
                .or_else(|| {
                    nutrients
                        .iter()
                        .find(|(name, _)| name.contains(alias))
                        .map(|(_, value)| *value)
                })
        })
        .unwrap_or(0.0)
}

fn round(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

use nutrisurvey_lib::{
    foods, meals,
    models::{AiMealInput, RecommendationFilter},
    nutrition,
};
use std::sync::atomic::{AtomicUsize, Ordering};

static DATABASE_ID: AtomicUsize = AtomicUsize::new(0);

async fn storage() -> nutrisurvey_lib::storage::Storage {
    let id = DATABASE_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "nutrisurvey-task4-{}-{}.sqlite",
        std::process::id(),
        id
    ));
    let _ = tokio::fs::remove_file(&path).await;
    nutrisurvey_lib::storage::Storage::open_path(&path)
        .await
        .unwrap()
}

#[test]
fn tdee_uses_clinical_reference_weight_for_obesity() {
    let response = nutrition::calculate_tdee(nutrisurvey_lib::models::TdeeRequest {
        weight_kg: 110.0,
        height_cm: 170.0,
        age: 30,
        gender: "Male".into(),
        activity_factor: 1.2,
        injury_factor: 1.25,
        is_manual_factors: false,
        bmi_standard: "asia_pacific".into(),
    })
    .unwrap();
    assert_eq!(response.bmi, 38.06);
    assert_eq!(response.nutrition_classification, "Obes morbid");
    assert_eq!(response.ideal_weight, 63.0);
    assert_eq!(response.adjusted_weight, 74.75);
    assert_eq!(response.reference_weight, 74.75);
}

#[test]
fn tdee_uses_actual_weight_for_normal_bmi() {
    let response = nutrition::calculate_tdee(nutrisurvey_lib::models::TdeeRequest {
        weight_kg: 70.0,
        height_cm: 175.0,
        age: 30,
        gender: "Male".into(),
        activity_factor: 1.2,
        injury_factor: 0.9,
        is_manual_factors: false,
        bmi_standard: "asia_pacific".into(),
    })
    .unwrap();
    assert_eq!(response.nutrition_classification, "Normal");
    assert_eq!(response.reference_weight, 70.0);
}

#[test]
fn tdee_uses_male_formula_and_factors() {
    let response = nutrition::calculate_tdee(nutrisurvey_lib::models::TdeeRequest {
        weight_kg: 70.0,
        height_cm: 175.0,
        age: 30,
        gender: "Male".into(),
        activity_factor: 1.2,
        injury_factor: 0.9,
        is_manual_factors: false,
        bmi_standard: "asia_pacific".into(),
    })
    .unwrap();
    assert_eq!(response.basal_metabolic_rate, 1696.0);
    assert_eq!(response.total_daily_energy_expenditure, 1831.68);
}

#[test]
fn tdee_uses_female_other_formula() {
    let response = nutrition::calculate_tdee(nutrisurvey_lib::models::TdeeRequest {
        weight_kg: 60.0,
        height_cm: 165.0,
        age: 25,
        gender: "other".into(),
        activity_factor: 1.5,
        injury_factor: 1.1,
        is_manual_factors: false,
        bmi_standard: "asia_pacific".into(),
    })
    .unwrap();
    assert_eq!(response.basal_metabolic_rate, 1410.5);
    assert_eq!(response.total_daily_energy_expenditure, 2327.33);
}

#[test]
fn tdee_rejects_invalid_numeric_input() {
    let result = nutrition::calculate_tdee(nutrisurvey_lib::models::TdeeRequest {
        weight_kg: f64::NAN,
        height_cm: 165.0,
        age: 25,
        gender: "female".into(),
        activity_factor: 1.5,
        injury_factor: 1.1,
        is_manual_factors: false,
        bmi_standard: "asia_pacific".into(),
    });
    assert!(result.is_err());
}

#[test]
fn tdee_rejects_non_positive_factors() {
    for factor in [0.0, -1.0] {
        let activity = nutrition::calculate_tdee(nutrisurvey_lib::models::TdeeRequest {
            weight_kg: 70.0,
            height_cm: 175.0,
            age: 30,
            gender: "male".into(),
            activity_factor: factor,
            injury_factor: 1.0,
            is_manual_factors: false,
            bmi_standard: "asia_pacific".into(),
        });
        let injury = nutrition::calculate_tdee(nutrisurvey_lib::models::TdeeRequest {
            weight_kg: 70.0,
            height_cm: 175.0,
            age: 30,
            gender: "male".into(),
            activity_factor: 1.0,
            injury_factor: factor,
            is_manual_factors: false,
            bmi_standard: "asia_pacific".into(),
        });
        assert!(activity.is_err());
        assert!(injury.is_err());
    }
}

#[tokio::test]
async fn recommendations_apply_combined_operators_and_cap() {
    let storage = storage().await;
    for food_id in 1..=12 {
        sqlx::query("INSERT INTO foods (id,name,normalized_name,serving_size,serving_unit,servings_per_container) VALUES (?,?,?,?,?,?)")
            .bind(food_id)
            .bind(format!("Food {food_id}"))
            .bind(format!("food {food_id}"))
            .bind(100.0)
            .bind("g")
            .bind(1.0)
            .execute(storage.pool()).await.unwrap();
    }
    for (id, name, amount) in [
        (1, "Protein", 30.0),
        (2, "Energy", 200.0),
        (3, "Carbohydrate", 10.0),
    ] {
        sqlx::query("INSERT INTO nutrients (id,name,normalized_name,unit) VALUES (?,?,?,?)")
            .bind(id)
            .bind(name)
            .bind(name.to_lowercase())
            .bind("g")
            .execute(storage.pool())
            .await
            .unwrap();
        for food_id in 1..=12 {
            sqlx::query("INSERT INTO food_nutrients (food_id,nutrient_id,amount) VALUES (?,?,?)")
                .bind(food_id)
                .bind(id)
                .bind(amount)
                .execute(storage.pool())
                .await
                .unwrap();
        }
    }
    let result = foods::recommend(
        &storage,
        &[
            RecommendationFilter {
                nutrient: "protein".into(),
                operator: ">".into(),
                value: 20.0,
            },
            RecommendationFilter {
                nutrient: "energy".into(),
                operator: "<".into(),
                value: 300.0,
            },
            RecommendationFilter {
                nutrient: "carbohydrate".into(),
                operator: "=".into(),
                value: 10.0,
            },
        ],
    )
    .await
    .unwrap();
    assert_eq!(result.len(), 12);
}

#[tokio::test]
async fn recommendations_reject_unsupported_operator() {
    let storage = storage().await;
    let result = foods::recommend(
        &storage,
        &[RecommendationFilter {
            nutrient: "protein".into(),
            operator: ">=".into(),
            value: 20.0,
        }],
    )
    .await;
    assert!(matches!(
        result,
        Err(nutrisurvey_lib::error::AppError::Validation(_))
    ));
}

#[tokio::test]
async fn recommendations_return_empty_for_missing_nutrient() {
    let storage = storage().await;
    sqlx::query("INSERT INTO foods (id,name,normalized_name,serving_size,serving_unit,servings_per_container) VALUES (1,'Food','food',100,'g',1)")
        .execute(storage.pool()).await.unwrap();
    let result = foods::recommend(
        &storage,
        &[RecommendationFilter {
            nutrient: "missing".into(),
            operator: ">".into(),
            value: 0.0,
        }],
    )
    .await
    .unwrap();
    assert!(result.is_empty());
}

#[test]
fn command_registration_exposes_tdee_command_contract() {
    let result = nutrisurvey_lib::commands::calculate_tdee(nutrisurvey_lib::models::TdeeRequest {
        weight_kg: 70.0,
        height_cm: 175.0,
        age: 30,
        gender: "male".into(),
        activity_factor: 1.0,
        injury_factor: 1.0,
        is_manual_factors: false,
        bmi_standard: "asia_pacific".into(),
    })
    .unwrap();
    assert_eq!(result.formula_used, "Harris-Benedict (Clinical Edition)");
}

#[tokio::test]
async fn meal_mapping_scales_nutrients_and_uses_match_priority() {
    let storage = storage().await;
    sqlx::query("INSERT INTO foods (id,name,normalized_name,serving_size,serving_unit,servings_per_container) VALUES (1,'Nasi','nasi',100,'g',1),(2,'Nasi Goreng','nasi goreng',100,'g',1)").execute(storage.pool()).await.unwrap();
    for (id, name, unit) in [
        (1, "Energi", "kcal"),
        (2, "Protein", "g"),
        (3, "Lemak Total", "g"),
        (4, "Karbohidrat Total", "g"),
    ] {
        sqlx::query("INSERT INTO nutrients (id,name,normalized_name,unit) VALUES (?,?,?,?)")
            .bind(id)
            .bind(name)
            .bind(name.to_lowercase())
            .bind(unit)
            .execute(storage.pool())
            .await
            .unwrap();
    }
    for (nutrient, amount) in [(1, 130.0), (2, 2.7), (3, 0.3), (4, 28.2)] {
        sqlx::query("INSERT INTO food_nutrients (food_id,nutrient_id,amount) VALUES (?,?,?)")
            .bind(1)
            .bind(nutrient)
            .bind(amount)
            .execute(storage.pool())
            .await
            .unwrap();
    }
    let mapped = meals::map_ai_items(
        &storage,
        &[AiMealInput {
            meal_type: "SARAPAN".into(),
            food_keyword: " NASI ".into(),
            suggested_grams: 200,
            reasoning: "match".into(),
        }],
    )
    .await
    .unwrap();
    assert_eq!(mapped[0].matched_food_id, 1);
    assert_eq!(mapped[0].suggested_grams, 200);
    assert_eq!(mapped[0].calories, 260.0);
    assert_eq!(mapped[0].protein, 5.4);
    assert_eq!(mapped[0].nutrients["energi"], 260.0);
}

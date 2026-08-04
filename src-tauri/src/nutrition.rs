use crate::{
    error::AppError,
    models::{TdeeRequest, TdeeResponse},
};

pub fn calculate_tdee(request: TdeeRequest) -> Result<TdeeResponse, AppError> {
    if !request.weight_kg.is_finite()
        || request.weight_kg <= 0.0
        || !request.height_cm.is_finite()
        || request.height_cm <= 0.0
        || request.age <= 0
        || !request.activity_factor.is_finite()
        || request.activity_factor <= 0.0
        || !request.injury_factor.is_finite()
        || request.injury_factor <= 0.0
    {
        return Err(AppError::Validation(
            "numeric input must be finite and positive".into(),
        ));
    }
    let male = request.gender.eq_ignore_ascii_case("male");
    let bmi = request.weight_kg / (request.height_cm / 100.0).powi(2);
    let nutrition_classification = classify_bmi(bmi, &request.bmi_standard).to_string();
    let ideal_weight = ideal_weight(request.height_cm, male);
    let adjusted_weight = ideal_weight + (request.weight_kg - ideal_weight) * 0.25;
    let reference_weight = if bmi > 30.0 {
        adjusted_weight
    } else if bmi >= 23.0 {
        ideal_weight
    } else {
        request.weight_kg
    };
    let bmr = if male {
        66.0 + 13.7 * reference_weight + 5.0 * request.height_cm - 6.8 * request.age as f64
    } else {
        655.0 + 9.6 * reference_weight + 1.8 * request.height_cm - 4.7 * request.age as f64
    };
    Ok(TdeeResponse {
        basal_metabolic_rate: round(bmr),
        total_daily_energy_expenditure: round(
            bmr * request.activity_factor * request.injury_factor,
        ),
        formula_used: "Harris-Benedict (Clinical Edition)".into(),
        bmi: round(bmi),
        nutrition_classification,
        ideal_weight: round(ideal_weight),
        adjusted_weight: round(adjusted_weight),
        reference_weight: round(reference_weight),
    })
}

fn classify_bmi(bmi: f64, standard: &str) -> &'static str {
    if standard == "who" {
        if bmi < 18.5 {
            "Underweight"
        } else if bmi < 25.0 {
            "Normal"
        } else if bmi < 30.0 {
            "Overweight"
        } else {
            "Obesitas"
        }
    } else {
        if bmi < 16.0 {
            "Malnutrisi berat"
        } else if bmi < 17.0 {
            "Malnutrisi sedang"
        } else if bmi < 18.5 {
            "Malnutrisi ringan"
        } else if bmi < 23.0 {
            "Normal"
        } else if bmi < 25.0 {
            "Overweight"
        } else if bmi <= 30.0 {
            "Obes grade I"
        } else if bmi <= 35.0 {
            "Obes grade II"
        } else {
            "Obes morbid"
        }
    }
}

fn ideal_weight(height_cm: f64, male: bool) -> f64 {
    let threshold = if male { 160.0 } else { 150.0 };
    if height_cm >= threshold {
        0.9 * (height_cm - 100.0)
    } else {
        height_cm - 100.0
    }
}

fn round(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

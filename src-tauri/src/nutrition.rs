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
        || !request.injury_factor.is_finite()
    {
        return Err(AppError::Validation(
            "numeric input must be finite and positive".into(),
        ));
    }
    let male = request.gender.eq_ignore_ascii_case("male");
    let bmr = if male {
        66.0 + 13.7 * request.weight_kg + 5.0 * request.height_cm - 6.8 * request.age as f64
    } else {
        655.0 + 9.6 * request.weight_kg + 1.8 * request.height_cm - 4.7 * request.age as f64
    };
    Ok(TdeeResponse {
        basal_metabolic_rate: round(bmr),
        total_daily_energy_expenditure: round(
            bmr * request.activity_factor * request.injury_factor,
        ),
        formula_used: "Harris-Benedict (Clinical Edition)".into(),
    })
}

fn round(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

use crate::{error::AppError, models::AiRequest};
use serde_json::Value;

pub(crate) fn build(request: &AiRequest) -> String {
    format!(
        "Buat rencana makan harian untuk target {} kkal.\nTarget makro absolut: Karbohidrat {}g, Protein {}g, Lemak {}g.\nWaktu makan WAJIB menggunakan kategori berikut SAJA: {}.\nBagilah makanan ke dalam kategori-kategori tersebut secara logis. DILARANG menggunakan istilah waktu makan lain di luar daftar tersebut.\nKembalikan hanya JSON valid dengan bentuk:\n{{\"meal_plan\":[{{\"meal_type\":\"Makan Pagi\",\"food_keyword\":\"Dada Ayam\",\"suggested_grams\":100,\"reasoning\":\"alasan singkat\"}}]}}\nfood_keyword harus berupa nama bahan/makanan dalam bahasa Indonesia untuk dicocokkan dengan database SQLite internal.\nJangan tambah markdown, komentar, atau teks di luar JSON.",
        request.target_tdee,
        request.target_carbs,
        request.target_protein,
        request.target_fat,
        request.available_meal_types.join(", "),
    )
}

pub(crate) fn parse_meal_plan(content: &str) -> Result<Vec<crate::models::AiMealInput>, AppError> {
    let json = strip_fence(content.trim());
    let value: Value = serde_json::from_str(json)
        .map_err(|_| AppError::Ai("AI meal plan JSON is malformed".into()))?;
    let plan = value
        .get("meal_plan")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::Ai("AI meal plan is missing meal_plan".into()))?;
    if plan.iter().any(|item| {
        !item.get("meal_type").and_then(Value::as_str).is_some()
            || !item.get("food_keyword").and_then(Value::as_str).is_some()
            || item
                .get("suggested_grams")
                .and_then(Value::as_i64)
                .is_none()
            || !item.get("reasoning").and_then(Value::as_str).is_some()
    }) {
        return Err(AppError::Ai("AI meal plan contains missing fields".into()));
    }
    serde_json::from_value(Value::Array(plan.clone()))
        .map_err(|_| AppError::Ai("AI meal plan fields are invalid".into()))
}

fn strip_fence(value: &str) -> &str {
    let Some(value) = value.strip_prefix("```") else {
        return value;
    };
    let value = value
        .strip_prefix("json")
        .unwrap_or(value)
        .trim_start_matches('\n');
    value.strip_suffix("```").unwrap_or(value).trim()
}

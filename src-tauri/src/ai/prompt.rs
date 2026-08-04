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

pub fn parse_meal_plan(content: &str) -> Result<Vec<crate::models::AiMealInput>, AppError> {
    let json = extract_json(content)?;
    let value: Value = serde_json::from_str(&json)
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

fn extract_json(content: &str) -> Result<String, AppError> {
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let candidate = normalized
        .lines()
        .enumerate()
        .find_map(|(index, line)| {
            let trimmed = line.trim();
            if trimmed.starts_with("```") && trimmed[3..].trim().eq_ignore_ascii_case("json") {
                Some(
                    normalized
                        .lines()
                        .skip(index + 1)
                        .take_while(|line| line.trim() != "```")
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            } else {
                None
            }
        })
        .unwrap_or(normalized);
    let start = candidate
        .find('{')
        .ok_or_else(|| AppError::Ai("AI meal plan JSON is malformed".into()))?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, character) in candidate[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
        } else if character == '"' {
            in_string = true;
        } else if character == '{' {
            depth += 1;
        } else if character == '}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Ok(candidate[start..start + offset + character.len_utf8()].to_string());
            }
        }
    }
    Err(AppError::Ai("AI meal plan JSON is malformed".into()))
}

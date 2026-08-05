use crate::{error::AppError, models::AiRequest};
use serde_json::Value;

pub(crate) fn build(request: &AiRequest) -> String {
    format!(
        "Anda adalah Senior Clinical Dietitian yang mengikuti PAGT/ADIME dan instruksi Nutrisionist Klinis. Data klinis terverifikasi dari Kalkulator TDEE adalah sumber utama untuk IMT, klasifikasi gizi, BB referensi, dan kebutuhan energi; jangan tanyakan ulang data tersebut. Untuk overweight/obesitas gunakan defisit aman; untuk malnutrisi perhatikan risiko refeeding; untuk diabetes, hipertensi, CKD, atau kondisi lain ikuti batasan klinis yang relevan. Gunakan asesmen FH (pola makan, alergi, pantangan, suka/tidak suka), CH (diagnosis, obat, akses bahan), BD (hasil lab bila relevan), dan PD (edema, massa otot/lemak, tanda defisiensi bila relevan). Jangan mengarang data yang tidak diberikan. Susun menu hanya setelah asesmen cukup, sesuaikan kondisi klinis, gunakan sistem penukar bahan makanan, dan sertakan alasan klinis singkat. Untuk CKD, diabetes, hipertensi, obesitas, atau malnutrisi ikuti batasan klinis yang relevan. Rencana ini edukatif dan bukan pengganti konsultasi dokter/ahli gizi.\nBuat rencana makan harian untuk target {} kkal.\nTarget makro absolut: Karbohidrat {}g, Protein {}g, Lemak {}g.\nInstruksi tambahan dan hasil asesmen user:\n{}\nWaktu makan WAJIB menggunakan kategori berikut SAJA: {}.\nBagilah makanan ke dalam kategori-kategori tersebut secara logis. DILARANG menggunakan istilah waktu makan lain di luar daftar tersebut.\nKembalikan hanya JSON valid dengan bentuk:\n{{\"meal_plan\":[{{\"meal_type\":\"Makan Pagi\",\"food_keyword\":\"Dada Ayam\",\"suggested_grams\":100,\"reasoning\":\"alasan singkat\"}}]}}\nfood_keyword harus berupa nama bahan/makanan dalam bahasa Indonesia untuk dicocokkan dengan database SQLite internal.\nJangan tambah markdown, komentar, atau teks di luar JSON.",
        request.target_tdee,
        request.target_carbs,
        request.target_protein,
        request.target_fat,
        if request.prompt.trim().is_empty() { "Tidak ada instruksi tambahan" } else { request.prompt.trim() },
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
    let mut items = Vec::new();
    for item in plan {
        let meal_type = item
            .get("meal_type")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_owned();
        let food_keyword = item
            .get("food_keyword")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_owned();
        let suggested_grams = item
            .get("suggested_grams")
            .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f.round() as i64)))
            .unwrap_or(0) as i32;
        let reasoning = item
            .get("reasoning")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if meal_type.is_empty() || food_keyword.is_empty() || suggested_grams <= 0 {
            continue;
        }
        items.push(crate::models::AiMealInput {
            meal_type,
            food_keyword,
            suggested_grams,
            reasoning,
        });
    }
    if items.is_empty() {
        return Err(AppError::Ai("AI meal plan returned no valid items".into()));
    }
    Ok(items)
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

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DEFAULT_MEALS: [(&str, &str); 3] = [
    ("SARAPAN", "Sarapan"),
    ("MAKAN_SIANG", "Makan Siang"),
    ("MAKAN_MALAM", "Makan Malam"),
];
const CONTENT_TYPE: &str = "application/rtf";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FoodEntry {
    pub meal_time: String,
    pub name: String,
    pub amount: f64,
    #[serde(default = "default_serving_size")]
    pub serving_size: f64,
    pub serving_unit: Option<String>,
    #[serde(default)]
    pub nutrients: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MealTime {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NutritionTargets {
    pub kcal: f64,
    pub carbs: f64,
    pub protein: f64,
    pub fat: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub foods: Vec<FoodEntry>,
    pub meal_times: Option<Vec<MealTime>>,
    pub targets: Option<NutritionTargets>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub filename: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
    pub saved_path: Option<String>,
}

fn default_serving_size() -> f64 {
    100.0
}

pub fn render_rtf(request: ExportRequest, template: &[u8]) -> Result<Vec<u8>, AppError> {
    let template = std::str::from_utf8(template)
        .map_err(|error| AppError::Export(format!("Template RTF tidak valid: {error}")))?;
    let start = template
        .find("=====================================================================")
        .ok_or_else(|| AppError::Export("Template RTF tidak valid".into()))?;
    let end_marker = "\\par }{\\*\\themedata";
    let end = template[start..]
        .find(end_marker)
        .map(|offset| start + offset)
        .ok_or_else(|| AppError::Export("Template RTF tidak valid".into()))?;

    let meals = request.meal_times.unwrap_or_else(|| {
        DEFAULT_MEALS
            .iter()
            .map(|(id, label)| MealTime {
                id: (*id).into(),
                label: (*label).into(),
            })
            .collect()
    });
    let mut totals = HashMap::new();
    let mut rows = Vec::new();
    for meal in meals {
        let mut foods = Vec::new();
        let mut energy = 0.0;
        let mut carbs = 0.0;
        for food in request
            .foods
            .iter()
            .filter(|food| food.meal_time == meal.id)
        {
            let base = if food.serving_size > 0.0 {
                food.serving_size
            } else {
                100.0
            };
            let ratio = if food.amount > 0.0 { food.amount } else { 0.0 } / base;
            let food_energy = get_aliases(&food.nutrients, &["energy", "energi"]) * ratio;
            let food_carbs = get_aliases(
                &food.nutrients,
                &[
                    "carbohydrate",
                    "carbohydr.",
                    "karbohidrat",
                    "karbohidrat total",
                    "carbs",
                    "karbo",
                ],
            ) * ratio;
            energy += food_energy;
            carbs += food_carbs;
            let unit = food
                .serving_unit
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("g");
            foods.push((
                food.name.as_str(),
                format_number(food.amount, 1, 0) + " " + unit,
                food_energy,
                food_carbs,
            ));
            for (name, value) in &food.nutrients {
                let key = name.trim().to_lowercase();
                if !key.is_empty() {
                    *totals.entry(key).or_insert(0.0) += value * ratio;
                }
            }
        }
        rows.push((meal.label, foods, energy, carbs));
    }
    let total_energy: f64 = rows.iter().map(|row| row.2).sum();
    let total_carbs: f64 = rows.iter().map(|row| row.3).sum();
    let mut body = String::from("=====================================================================\\par \\pard \\ltrpar\\qc \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\ab\\af0\\afs30 \\ltrch\\fcs0 \\b\\f0\\fs30\\kerning0 HASIL PERHITUNGAN DIET/\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 ");
    body.push_str("=====================================================================\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\tqc\\tx5500\\tqc\\tx7100\\tqc\\tx8400\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 Nama Makanan\\tab Jumlah\\tab energy\\tab carbohydr.\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 ______________________________________________________________________________ \\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\tqdec\\tx5500\\tqdec\\tx7100\\tqdec\\tx8400\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\par }");
    for (label, foods, energy, carbs) in &rows {
        body.push_str("{\\rtlch\\fcs1 \\ab\\af0 \\ltrch\\fcs0 \\b\\f0\\kerning0 ");
        body.push_str(&escape_rtf(&label.to_uppercase()));
        body.push_str("}{\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\par ");
        for (name, amount, food_energy, food_carbs) in foods {
            body.push_str("\\hich\\af0\\dbch\\af31505\\loch\\f0 ");
            body.push_str(&escape_rtf(name));
            body.push_str("\\tab ");
            body.push_str(&escape_rtf(amount));
            body.push_str("\\tab ");
            body.push_str(&format_number(*food_energy, 1, 8));
            body.push_str(" kcal\\tab ");
            body.push_str(&format_number(*food_carbs, 1, 7));
            body.push_str("  g\\par ");
        }
        let energy_pct = if total_energy > 0.0 {
            energy / total_energy * 100.0
        } else {
            0.0
        };
        let carbs_pct = if total_carbs > 0.0 {
            carbs / total_carbs * 100.0
        } else {
            0.0
        };
        body.push_str(&format!("\\par \\hich\\af0\\dbch\\af31505\\loch\\f0 Meal analysis:  energy {} kcal ({} %),  carbohydrate {} g ({} %)\\par \\par \\par }}", format_number(*energy, 1, 0), format_number(energy_pct, 0, 0), format_number(*carbs, 1, 0), format_number(carbs_pct, 0, 0)));
    }
    body.push_str("\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\par \\hich\\af0\\dbch\\af31505\\loch\\f0 =====================================================================\\par }\\pard \\ltrpar\\qc \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\ab\\af0\\afs30 \\ltrch\\fcs0 \\b\\f0\\fs30\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 HASIL PERHITUNGAN\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 =====================================================================\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\tqc\\tx2900\\tqc\\tx5600\\tqc\\tx8300\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 Zat Gizi\\tab hasil analisis\\tab rekomendasi\\tab persentase\\par \\hich\\af0\\dbch\\af31505\\loch\\f0      \\tab nilai\\tab nilai/hari\\tab pemenuhan\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 ______________________________________________________________________________\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\tqdec\\tx3000\\tqdec\\tx5700\\tqdec\\tx8400\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 ");
    for row in nutrient_rows(&totals, request.targets.as_ref(), total_energy) {
        body.push_str(&escape_rtf(&row));
    }
    body.push('}');
    let mut output = String::with_capacity(template.len() + body.len());
    output.push_str(&template[..start]);
    output.push_str(&body);
    output.push_str(&template[end + "\\par }".len()..]);
    Ok(output.into_bytes())
}

pub fn result(bytes: Vec<u8>) -> ExportResult {
    ExportResult {
        filename: format!("Laporan_Nutrisi_{}.rtf", today()),
        content_type: CONTENT_TYPE.into(),
        bytes,
        saved_path: None,
    }
}

fn escape_rtf(value: &str) -> String {
    let mut out = String::new();
    for unit in value.encode_utf16() {
        let signed = unit as i32;
        if signed < 128 {
            match char::from_u32(signed as u32).unwrap() {
                '\\' => out.push_str("\\\\"),
                '{' => out.push_str("\\{"),
                '}' => out.push_str("\\}"),
                ch => out.push(ch),
            }
        } else {
            out.push_str(&format!(
                "\\u{}?",
                if signed > 32767 {
                    signed - 65536
                } else {
                    signed
                }
            ));
        }
    }
    out
}
fn normalize(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
fn get_aliases(values: &HashMap<String, f64>, aliases: &[&str]) -> f64 {
    let aliases: Vec<_> = aliases.iter().map(|value| normalize(value)).collect();
    let exact: f64 = values
        .iter()
        .filter(|(key, _)| aliases.contains(&normalize(key)))
        .map(|(_, value)| *value)
        .sum();
    if exact != 0.0 {
        return exact;
    }
    values
        .iter()
        .filter(|(key, _)| {
            aliases
                .iter()
                .any(|alias| normalize(key).contains(alias) || alias.contains(&normalize(key)))
        })
        .map(|(_, value)| *value)
        .sum()
}
fn format_number(value: f64, decimals: usize, width: usize) -> String {
    let mut text = format!("{value:.decimals$}").replace('.', ",");
    if width > text.len() {
        text = format!("{:>width$}", text, width = width);
    }
    text
}
fn nutrient_rows(
    totals: &HashMap<String, f64>,
    targets: Option<&NutritionTargets>,
    total_energy: f64,
) -> Vec<String> {
    let target = |value: f64, fallback: f64| if value > 0.0 { value } else { fallback };
    let rows = [
        (
            "energy",
            "kcal",
            target(targets.map_or(0.0, |t| t.kcal), 1900.0),
            ["energy", "energi"].as_slice(),
        ),
        ("water", "g", 2600.0, &["water", "air"]),
        (
            "protein",
            "g",
            target(targets.map_or(0.0, |t| t.protein), 47.0),
            &["protein"],
        ),
        (
            "fat",
            "g",
            target(targets.map_or(0.0, |t| t.fat), 73.0),
            &["fat", "lemak"],
        ),
        (
            "carbohydr.",
            "g",
            target(targets.map_or(0.0, |t| t.carbs), 332.0),
            &[
                "carbohydrate",
                "carbohydr.",
                "karbohidrat",
                "karbohidrat total",
                "carbs",
                "karbo",
            ],
        ),
        (
            "dietary fiber",
            "g",
            30.0,
            &["dietary fiber", "fiber", "serat"],
        ),
        ("alcohol", "g", 0.0, &["alcohol"]),
        ("PUFA", "g", 10.0, &["pufa"]),
        ("cholesterol", "mg", 0.0, &["cholesterol"]),
        ("Vit. A", "ug", 800.0, &["vit. a", "vitamin a"]),
        ("carotene", "mg", 0.0, &["carotene"]),
        ("Vit. E", "mg", 0.0, &["vit. e", "vitamin e"]),
        ("Vit. B1", "mg", 1.0, &["vit. b1", "vitamin b1", "thiamin"]),
        (
            "Vit. B2",
            "mg",
            1.2,
            &["vit. b2", "vitamin b2", "riboflavin"],
        ),
        ("Vit. B6", "mg", 1.2, &["vit. b6", "vitamin b6"]),
        ("folic acid eq.", "ug", 0.0, &["folic acid eq.", "folate"]),
        ("Vit. C", "mg", 100.0, &["vit. c", "vitamin c"]),
        ("sodium", "mg", 2000.0, &["sodium", "natrium"]),
        ("potassium", "mg", 3500.0, &["potassium", "kalium"]),
        ("calcium", "mg", 1000.0, &["calcium", "kalsium"]),
        ("magnesium", "mg", 300.0, &["magnesium"]),
        ("phosphorus", "mg", 700.0, &["phosphorus", "fosfor"]),
        ("iron", "mg", 15.0, &["iron", "zat besi"]),
        ("zinc", "mg", 7.0, &["zinc", "seng"]),
    ];
    rows.iter()
        .map(|(name, unit, recommendation_value, aliases)| {
            let value = get_aliases(totals, aliases);
            let macro_pct = if total_energy > 0.0 {
                match *name {
                    "protein" | "carbohydr." => value * 4.0 / total_energy * 100.0,
                    "fat" => value * 9.0 / total_energy * 100.0,
                    _ => 0.0,
                }
            } else {
                0.0
            };
            let analysis = if ["protein", "fat", "carbohydr."].contains(name) {
                format!("{} {}({:.0}%)", format_number(value, 1, 0), unit, macro_pct)
            } else {
                format!("{} {}", format_number(value, 1, 0), unit)
            };
            let recommendation = if *recommendation_value > 0.0 {
                format!("{} {}", format_number(*recommendation_value, 1, 0), unit)
            } else {
                "-".into()
            };
            let percent = if *recommendation_value > 0.0 {
                format!("{:.0} %", value / recommendation_value * 100.0)
            } else {
                "-".into()
            };
            format!(
                "{}\\tab {}\\tab {}\\tab {} \\par ",
                name,
                analysis.replace(" ug", " \\hich\\f0 \'b5\\loch\\f0 g"),
                recommendation.replace(" ug", " \\hich\\f0 \'b5\\loch\\f0 g"),
                percent
            )
        })
        .collect()
}
fn today() -> String {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        / 86_400;
    let (year, month, day) = civil_date(days as i64);
    format!("{year:04}{month:02}{day:02}")
}
fn civil_date(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    (y as i32 + if m <= 2 { 1 } else { 0 }, m as u32, d as u32)
}

pub fn content_type() -> &'static str {
    CONTENT_TYPE
}

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DEFAULT_MEALS: [(&str, &str); 3] = [
    ("SARAPAN", "Sarapan"),
    ("MAKAN_SIANG", "Makan Siang"),
    ("MAKAN_MALAM", "Makan Malam"),
];
/// Format ekspor laporan yang dipilih pengguna (PDF / DOCX / RTF).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Pdf,
    Docx,
    Rtf,
}

impl ExportFormat {
    /// Parse string format dari frontend, tidak peka huruf besar/kecil.
    pub fn parse(value: &str) -> Result<ExportFormat, AppError> {
        match value.trim().to_lowercase().as_str() {
            "pdf" => Ok(ExportFormat::Pdf),
            "docx" => Ok(ExportFormat::Docx),
            "rtf" => Ok(ExportFormat::Rtf),
            other => Err(AppError::Export(format!(
                "Format ekspor tidak didukung: {other}"
            ))),
        }
    }
    /// Ekstensi berkas keluaran tanpa titik.
    pub fn extension(self) -> &'static str {
        match self {
            ExportFormat::Pdf => "pdf",
            ExportFormat::Docx => "docx",
            ExportFormat::Rtf => "rtf",
        }
    }
    /// MIME type sesuai format.
    pub fn content_type(self) -> &'static str {
        match self {
            ExportFormat::Pdf => "application/pdf",
            ExportFormat::Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
            ExportFormat::Rtf => "application/rtf",
        }
    }
    /// Nama manusiawi untuk filter dialog penyimpanan.
    pub fn label(self) -> &'static str {
        match self {
            ExportFormat::Pdf => "PDF Document",
            ExportFormat::Docx => "Word Document",
            ExportFormat::Rtf => "Rich Text Format",
        }
    }
}

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

/// Permintaan ekspor dari frontend; field camelCase agar `mealTimes` dan
/// `targets` tidak terbuang saat deserialisasi (Option yang hilang = None).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    pub delivery: ExportDelivery,
    pub saved_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExportDelivery {
    Share,
    Saved,
}

fn default_serving_size() -> f64 {
    100.0
}

/// Satu baris makanan pada tabel per-waktu-makan.
pub struct ReportFoodRow {
    pub name: String,
    pub amount: String,
    pub energy: f64,
    pub carbs: f64,
}

/// Ringkasan satu waktu makan beserta analisis energinya.
pub struct ReportMeal {
    pub label: String,
    pub foods: Vec<ReportFoodRow>,
    pub energy: f64,
    pub carbs: f64,
}

/// Model perantara laporan yang dipakai bersama oleh renderer RTF, DOCX, dan
/// PDF agar ketiga format selalu menampilkan angka yang sama dengan Dashboard.
pub struct Report {
    pub meals: Vec<ReportMeal>,
    pub total_energy: f64,
    pub total_carbs: f64,
    pub nutrient_totals: HashMap<String, f64>,
}

/// Susun model laporan dari permintaan ekspor.
///
/// Nilai makro per makanan dihitung dengan semantik yang sama persis dengan
/// `getMacroValue` pada Dashboard frontend: alias dicocokkan secara eksak
/// (kunci normalisasi), urutan alias menentukan pemenang, dan nilai TIDAK
/// dijumlahkan antar kunci yang mirip agar tidak terjadi double counting.
pub fn build_report(request: &ExportRequest) -> Report {
    // meal_times yang hilang (payload lama/tidak lengkap) diturunkan dari
    // foods agar laporan tidak menjadi kosong; hanya bila tidak ada foods
    // pun dipakai DEFAULT_MEALS.
    let meals = request.meal_times.clone().unwrap_or_else(|| {
        let mut ids: Vec<String> = Vec::new();
        for food in &request.foods {
            if !food.meal_time.is_empty() && !ids.contains(&food.meal_time) {
                ids.push(food.meal_time.clone());
            }
        }
        if ids.is_empty() {
            DEFAULT_MEALS
                .iter()
                .map(|(id, label)| MealTime {
                    id: (*id).into(),
                    label: (*label).into(),
                })
                .collect()
        } else {
            ids.into_iter()
                .map(|id| MealTime {
                    label: id.clone(),
                    id,
                })
                .collect()
        }
    });
    let mut nutrient_totals = HashMap::new();
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
            // Alias dan urutannya sama persis dengan macroMap di Dashboard
            // (src/lib/types.ts) agar nilai laporan identik dengan Dashboard.
            let food_energy = get_aliases(
                &food.nutrients,
                &[
                    "energi",
                    "energy",
                    "energy (kcal)",
                    "energi (kkal)",
                    "energi total",
                    "total energy",
                ],
            ) * ratio;
            let food_carbs = get_aliases(
                &food.nutrients,
                &[
                    "karbohidrat total",
                    "karbohidrat",
                    "carbohydrate",
                    "carbohydrates",
                    "total carbohydrate",
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
            foods.push(ReportFoodRow {
                name: food.name.clone(),
                amount: format_number(food.amount, 1, 0) + " " + unit,
                energy: food_energy,
                carbs: food_carbs,
            });
            for (name, value) in &food.nutrients {
                let key = name.trim().to_lowercase();
                if !key.is_empty() {
                    *nutrient_totals.entry(key).or_insert(0.0) += value * ratio;
                }
            }
        }
        rows.push(ReportMeal {
            label: meal.label,
            foods,
            energy,
            carbs,
        });
    }
    let total_energy: f64 = rows.iter().map(|row| row.energy).sum();
    let total_carbs: f64 = rows.iter().map(|row| row.carbs).sum();
    Report {
        meals: rows,
        total_energy,
        total_carbs,
        nutrient_totals,
    }
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

    let report = build_report(&request);
    let total_energy = report.total_energy;
    let total_carbs = report.total_carbs;
    let mut body = String::from("=====================================================================\\par \\pard \\ltrpar\\qc \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\ab\\af0\\afs30 \\ltrch\\fcs0 \\b\\f0\\fs30\\kerning0 HASIL PERHITUNGAN DIET\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 ");
    body.push_str("=====================================================================\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\tqc\\tx5500\\tqc\\tx7100\\tqc\\tx8400\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 Nama Makanan\\tab Jumlah\\tab energy\\tab carbohydr.\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 ______________________________________________________________________________ \\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\tqdec\\tx5500\\tqdec\\tx7100\\tqdec\\tx8400\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\par }");
    for (label, foods, energy, carbs) in report
        .meals
        .iter()
        .map(|meal| (&meal.label, &meal.foods, meal.energy, meal.carbs))
    {
        body.push_str("{\\rtlch\\fcs1 \\ab\\af0 \\ltrch\\fcs0 \\b\\f0\\kerning0 ");
        body.push_str(&escape_rtf(label));
        body.push_str("}{\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\par ");
        for ReportFoodRow {
            name,
            amount,
            energy: food_energy,
            carbs: food_carbs,
        } in foods
        {
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
        body.push_str(&format!("\\par \\hich\\af0\\dbch\\af31505\\loch\\f0 Meal analysis:  energy {} kcal ({} %),  carbohydrate {} g ({} %)\\par \\par \\par }}", format_number(energy, 1, 0), format_number(energy_pct, 0, 0), format_number(carbs, 1, 0), format_number(carbs_pct, 0, 0)));
    }
    // Halaman baru sebelum bagian HASIL PERHITUNGAN, seperti tata letak contoh.
    body.push_str("\\page ");
    body.push_str("\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\par \\hich\\af0\\dbch\\af31505\\loch\\f0 =====================================================================\\par }\\pard \\ltrpar\\qc \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\ab\\af0\\afs30 \\ltrch\\fcs0 \\b\\f0\\fs30\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 HASIL PERHITUNGAN\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 =====================================================================\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\tqc\\tx3000\\tqc\\tx5700\\tqc\\tx8400\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 Zat Gizi\\tab hasil analisis\\tab rekomendasi\\tab persentase\\par \\hich\\af0\\dbch\\af31505\\loch\\f0      \\tab nilai\\tab nilai/hari\\tab pemenuhan\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 ______________________________________________________________________________\\par }\\pard \\ltrpar\\ql \\li0\\ri0\\nowidctlpar\\tqdec\\tx3000\\tqdec\\tx5700\\tqdec\\tx8400\\wrapdefault\\faauto\\rin0\\lin0\\itap0 {\\rtlch\\fcs1 \\af0 \\ltrch\\fcs0 \\f0\\kerning0 \\hich\\af0\\dbch\\af31505\\loch\\f0 ");
    for row in nutrient_rows(
        &report.nutrient_totals,
        request.targets.as_ref(),
        total_energy,
    ) {
        body.push_str(&row);
    }
    // Template dibelah di region yang net-nya -1 (menutup dua group dari prefix
    // dan satu dari tail). Body harus menutup keduanya, jika tidak RTF jadi
    // tidak seimbang dan Word menampilkan dokumen kosong.
    body.push('}');
    body.push('}');
    let mut output = String::with_capacity(template.len() + body.len());
    output.push_str(&template[..start]);
    output.push_str(&body);
    output.push_str(&template[end + "\\par }".len()..]);
    // Baris aturan (===== / _____) dibungkus \fs24 (12 pt) agar memenuhi
    // lebar halaman seperti contoh asli.
    let output = wrap_rules(&output);
    Ok(output.into_bytes())
}

pub fn result(kind: ExportFormat, bytes: Vec<u8>) -> ExportResult {
    ExportResult {
        filename: format!("Laporan_Nutrisi_{}.{}", today(), kind.extension()),
        content_type: kind.content_type().into(),
        bytes,
        delivery: ExportDelivery::Share,
        saved_path: None,
    }
}

/// Bungkus baris aturan (`=====` / `_____`, minimal 60 karakter berurutan)
/// dengan `\fs24` (12 pt) agar memenuhi lebar halaman seperti contoh asli.
fn wrap_rules(output: &str) -> String {
    let mut out = String::with_capacity(output.len() + 64);
    let chars: Vec<char> = output.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if ch == '=' || ch == '_' {
            let mut run = 0;
            while index + run < chars.len() && chars[index + run] == ch {
                run += 1;
            }
            if run >= 60 {
                out.push_str("\\fs24 ");
                for _ in 0..run {
                    out.push(ch);
                }
                out.push_str("\\fs20 ");
            } else {
                for _ in 0..run {
                    out.push(ch);
                }
            }
            index += run;
        } else {
            out.push(ch);
            index += 1;
        }
    }
    out
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
                '\r' => out.push_str("\\line "),
                '\n' => out.push_str("\\line "),
                '\t' => out.push_str("\\tab "),
                ch if ch.is_control() => out.push_str(&format!("\\u{}?", signed)),
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
/// Ambil nilai nutrien dengan semantik yang sama dengan `getMacroValue` di
/// Dashboard frontend: cocokkan alias secara eksak terhadap kunci ternormalisasi
/// sesuai urutan alias (pemenang pertama), lalu fallback pencocokan mengandung
/// sebagai jaring pengaman. Nilai TIDAK pernah dijumlahkan antar kunci agar
/// angka laporan tidak berlipat seperti pada Dashboard.
fn get_aliases(values: &HashMap<String, f64>, aliases: &[&str]) -> f64 {
    let aliases: Vec<String> = aliases.iter().map(|value| normalize(value)).collect();
    let normalized: Vec<(String, f64)> = values
        .iter()
        .map(|(key, value)| (normalize(key), *value))
        .filter(|(key, _)| !key.is_empty())
        .collect();
    for alias in &aliases {
        if let Some((_, value)) = normalized.iter().find(|(key, _)| key == alias) {
            return *value;
        }
    }
    normalized
        .iter()
        .find(|(key, _)| {
            aliases
                .iter()
                .any(|alias| key.contains(alias) || alias.contains(key))
        })
        .map(|(_, value)| *value)
        .unwrap_or(0.0)
}
fn format_number(value: f64, decimals: usize, width: usize) -> String {
    let value = if value.abs() < 0.5 * 10_f64.powi(-(decimals as i32)) {
        0.0
    } else {
        value
    };
    let mut text = format!("{value:.decimals$}").replace('.', ",");
    if width > text.len() {
        text = format!("{:>width$}", text, width = width);
    }
    text
}
fn nutrient_columns(
    totals: &HashMap<String, f64>,
    targets: Option<&NutritionTargets>,
    total_energy: f64,
) -> Vec<(String, String, String, String)> {
    // Angka rekomendasi HARUS berasal dari target project (dashboard); tanpa
    // fallback AKG buatan agar laporan tidak menampilkan angka rekaan yang
    // tidak ada di project. Target 0 ditampilkan sebagai "-".
    let rows = [
        (
            "energy",
            "kcal",
            targets.map_or(1900.0, |t| t.kcal),
            ["energy", "energi"].as_slice(),
        ),
        ("water", "g", 2600.0, &["water", "air"]),
        (
            "protein",
            "g",
            targets.map_or(47.0, |t| t.protein),
            &["protein"],
        ),
        (
            "fat",
            "g",
            targets.map_or(73.0, |t| t.fat),
            &["lemak total", "lemak", "fat", "total fat", "fats"],
        ),
        (
            "carbohydr.",
            "g",
            targets.map_or(332.0, |t| t.carbs),
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
            let macro_pct = if macro_pct.abs() < 0.5 {
                0.0
            } else {
                macro_pct
            };
            let analysis = if ["protein", "fat", "carbohydr."].contains(name) {
                format!("{} {}({:.0}%)", format_number(value, 1, 0), unit, macro_pct)
            } else {
                format!("{} {}", format_number(value, 1, 0), unit)
            };
            let mut recommendation = if *recommendation_value > 0.0 {
                format!("{} {}", format_number(*recommendation_value, 1, 0), unit)
            } else {
                "-".into()
            };
            // Label kontribusi energi pada kolom rekomendasi, seperti template
            // asli: protein "(12 %)", fat "(< 30 %)", carbohydr. "(> 55 %)".
            let macro_label = match *name {
                "protein" => targets
                    .filter(|target| target.kcal > 0.0 && target.protein > 0.0)
                    .map(|target| format!("({:.0} %)", target.protein * 4.0 / target.kcal * 100.0)),
                "fat" => Some("(< 30 %)".to_string()),
                "carbohydr." => Some("(> 55 %)".to_string()),
                _ => None,
            };
            if recommendation != "-" {
                if let Some(label) = macro_label {
                    recommendation.push_str(&label);
                }
            }
            let percent = if *recommendation_value > 0.0 {
                format!("{:.0} %", value / recommendation_value * 100.0)
            } else {
                "-".into()
            };
            ((*name).to_string(), analysis, recommendation, percent)
        })
        .collect()
}

/// Format kolom nutrien menjadi baris RTF dengan tab dan simbol mikro (µ).
fn nutrient_rows(
    totals: &HashMap<String, f64>,
    targets: Option<&NutritionTargets>,
    total_energy: f64,
) -> Vec<String> {
    nutrient_columns(totals, targets, total_energy)
        .into_iter()
        .map(|(name, analysis, recommendation, percent)| {
            format!(
                "{}\\tab {}\\tab {}\\tab {} \\par ",
                name,
                analysis.replace(" ug", " \\hich\\f0 \\'b5\\loch\\f0 g"),
                recommendation.replace(" ug", " \\hich\\f0 \\'b5\\loch\\f0 g"),
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

pub fn resolve_selected_path<P, E>(selected: Option<Result<P, E>>) -> Result<Option<P>, AppError>
where
    E: std::fmt::Display,
{
    selected
        .map(|path| path.map_err(|error| AppError::Export(error.to_string())))
        .transpose()
}

pub fn validate_selected_path(
    path: Option<&std::path::Path>,
    kind: ExportFormat,
) -> Result<&std::path::Path, AppError> {
    let path = path.ok_or_else(|| AppError::Export("Penyimpanan dibatalkan".into()))?;
    let expected = kind.extension();
    if path.extension().and_then(|value| value.to_str()) != Some(expected) {
        return Err(AppError::Export(format!(
            "Lokasi penyimpanan harus berakhiran .{expected}"
        )));
    }
    Ok(path)
}

pub fn mobile_delivery_error() -> Result<ExportResult, AppError> {
    Err(AppError::Export(
        "Ekspor mobile belum didukung: plugin share/document-picker belum tersedia".into(),
    ))
}

pub fn mobile_delivery_result(_bytes: Vec<u8>) -> Result<ExportResult, AppError> {
    mobile_delivery_error()
}

/// Render laporan dalam format apa pun yang didukung.
pub fn render(kind: ExportFormat, request: ExportRequest) -> Result<Vec<u8>, AppError> {
    match kind {
        ExportFormat::Pdf => render_pdf(request),
        ExportFormat::Docx => render_docx(request),
        ExportFormat::Rtf => render_rtf(request, include_bytes!("../../Assets/template.rtf")),
    }
}

// ---------------------------------------------------------------------------
// DOCX (Office Open XML) — ZIP ber-metode STORE tanpa dependensi eksternal.
// ---------------------------------------------------------------------------

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Times New Roman meniru font body template RTF aplikasi NutriSurvey asli.
const DOCX_FONT: &str =
    r#"<w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/>"#;

fn docx_paragraph(text: &str, bold: bool, size_half_points: u32) -> String {
    let mut rpr = String::new();
    if bold {
        rpr.push_str("<w:b/>");
    }
    rpr.push_str(DOCX_FONT);
    rpr.push_str(&format!(r#"<w:sz w:val="{size_half_points}"/>"#));
    format!(
        "<w:p><w:r><w:rPr>{rpr}</w:rPr><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
        xml_escape(text)
    )
}

/// Paragraf judul rata tengah, meniru posisi judul pada template asli.
fn docx_centered(text: &str, size_half_points: u32) -> String {
    let p = docx_paragraph(text, true, size_half_points);
    p.replace(
        "<w:p><w:r>",
        "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r>",
    )
}

/// Pindah ke halaman baru (sebelum bagian HASIL PERHITUNGAN).
fn docx_page_break() -> String {
    "<w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>".to_string()
}

/// Baris aturan (=====) pada 12 pt agar memenuhi lebar halaman seperti contoh.
fn docx_rule(text: &str) -> String {
    docx_paragraph(text, false, 24)
}

fn docx_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let widths = [4600u32, 2200, 2600, 2600];
    let mut out = String::from(
        "<w:tbl><w:tblPr><w:tblW w:w=\"12000\" w:type=\"dxa\"/><w:tblBorders>\
<w:top w:val=\"single\" w:sz=\"4\" w:color=\"AAAAAA\"/>\
<w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"AAAAAA\"/>\
<w:left w:val=\"single\" w:sz=\"4\" w:color=\"AAAAAA\"/>\
<w:right w:val=\"single\" w:sz=\"4\" w:color=\"AAAAAA\"/>\
<w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"CCCCCC\"/>\
<w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"CCCCCC\"/>\
</w:tblBorders></w:tblPr>",
    );
    let cell = |text: &str, bold: bool, width: u32| {
        let rpr = format!(
            "<w:rPr>{}{DOCX_FONT}</w:rPr>",
            if bold { "<w:b/>" } else { "" }
        );
        format!(
            "<w:tc><w:tcPr><w:tcW w:w=\"{width}\" w:type=\"dxa\"/></w:tcPr>\
<w:p><w:r>{rpr}<w:t xml:space=\"preserve\">{}</w:t></w:r></w:p></w:tc>",
            xml_escape(text)
        )
    };
    out.push_str("<w:tr>");
    for (index, header) in headers.iter().enumerate() {
        out.push_str(&cell(header, true, widths[index.min(widths.len() - 1)]));
    }
    out.push_str("</w:tr>");
    for row in rows {
        out.push_str("<w:tr>");
        for (index, value) in row.iter().enumerate() {
            out.push_str(&cell(value, false, widths[index.min(widths.len() - 1)]));
        }
        out.push_str("</w:tr>");
    }
    out.push_str("</w:tbl>");
    out
}

/// Renderer DOCX: menghasilkan .docx (OOXML dalam ZIP STORE) dari model laporan.
pub fn render_docx(request: ExportRequest) -> Result<Vec<u8>, AppError> {
    let report = build_report(&request);
    let targets = request.targets;
    let separator = "=".repeat(69);
    let mut body = String::new();
    body.push_str(&docx_centered("HASIL PERHITUNGAN DIET", 30));
    body.push_str(&docx_rule(&separator));

    let food_headers = ["Nama Makanan", "Jumlah", "energy", "carbohydr."];
    for meal in &report.meals {
        body.push_str(&docx_paragraph(&meal.label, true, 24));
        let rows: Vec<Vec<String>> = meal
            .foods
            .iter()
            .map(|food| {
                vec![
                    food.name.clone(),
                    food.amount.clone(),
                    format!("{} kcal", format_number(food.energy, 1, 0)),
                    format!("{} g", format_number(food.carbs, 1, 0)),
                ]
            })
            .collect();
        body.push_str(&docx_table(&food_headers, &rows));
        let energy_pct = if report.total_energy > 0.0 {
            meal.energy / report.total_energy * 100.0
        } else {
            0.0
        };
        let carbs_pct = if report.total_carbs > 0.0 {
            meal.carbs / report.total_carbs * 100.0
        } else {
            0.0
        };
        body.push_str(&docx_paragraph(
            &format!(
                "Meal analysis: energy {} kcal ({} %), carbohydrate {} g ({} %)",
                format_number(meal.energy, 1, 0),
                format_number(energy_pct, 0, 0),
                format_number(meal.carbs, 1, 0),
                format_number(carbs_pct, 0, 0)
            ),
            false,
            20,
        ));
        body.push_str(&docx_paragraph("", false, 20));
    }

    body.push_str(&docx_paragraph(&separator, false, 20));
    // Halaman baru sebelum bagian HASIL PERHITUNGAN, seperti tata letak contoh.
    body.push_str(&docx_page_break());
    body.push_str(&docx_centered("HASIL PERHITUNGAN", 30));
    body.push_str(&docx_rule(&separator));
    let nutrient_headers = [
        "Zat Gizi",
        "hasil analisis nilai",
        "rekomendasi nilai/hari",
        "persentase pemenuhan",
    ];
    let nutrient_table: Vec<Vec<String>> = nutrient_table_rows(&report, targets.as_ref())
        .iter()
        .map(|(name, analysis, recommendation, percent)| {
            vec![
                name.clone(),
                analysis.replace("µ", "u"),
                recommendation.replace("µ", "u"),
                percent.clone(),
            ]
        })
        .collect();
    body.push_str(&docx_table(&nutrient_headers, &nutrient_table));

    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
<w:body>{body}<w:sectPr/></w:body></w:document>"
    );
    let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
<Default Extension=\"xml\" ContentType=\"application/xml\"/>\
<Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
</Types>";
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/>\
</Relationships>";
    Ok(zip_store(&[
        ("[Content_Types].xml", content_types.as_bytes()),
        ("_rels/.rels", rels.as_bytes()),
        ("word/document.xml", document.as_bytes()),
    ]))
}

/// Baris tabel nutrien (nama, hasil analisis, rekomendasi, persentase) untuk
/// renderer DOCX/PDF — hasilnya identik dengan tabel RTF.
fn nutrient_table_rows(
    report: &Report,
    targets: Option<&NutritionTargets>,
) -> Vec<(String, String, String, String)> {
    nutrient_columns(&report.nutrient_totals, targets, report.total_energy)
        .into_iter()
        .map(|(name, analysis, recommendation, percent)| {
            let plain = |value: String| value.replace(" ug", " µg");
            (name, plain(analysis), plain(recommendation), percent)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// PDF — minimal PDF 1.4, font Times (WinAnsi), A4, tanpa dependensi.
// ---------------------------------------------------------------------------

const PDF_PAGE_LINES: usize = 50;
const PDF_TOP: f64 = 760.0;
const PDF_LEADING: f64 = 13.8;
const PDF_MARGIN: f64 = 56.6;

/// Escape teks PDF (literal string); µ dikirim sebagai escape oktal `\265`
/// agar tampil benar dengan WinAnsiEncoding seperti contoh asli (µg).
fn pdf_escape_text(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            'µ' => out.push_str("\\265"),
            ch if (ch as u32) < 128 => out.push(ch),
            _ => out.push('?'),
        }
    }
    out
}

/// Batas karakter per kolom; kolom diposisikan lewat koordinat-x tetap.
fn pdf_truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

/// Posisi x (poin) kolom tabel makanan: nama, jumlah, energy, carbohydr.
/// Sama seperti contoh asli: 56,6 / 312,1 / 395,5 / 451,0.
const PDF_FOOD_COLUMNS: [f64; 4] = [56.6, 312.1, 395.5, 451.0];
/// Posisi x (poin) kolom tabel nutrien (halaman HASIL PERHITUNGAN):
/// 56,6 / 180,7 / 302,7 / 445,8 seperti contoh asli.
const PDF_NUTRIENT_COLUMNS: [f64; 4] = [56.6, 180.7, 302.7, 445.8];

/// Satu baris PDF: segmen teks dengan koordinat-x masing-masing.
struct PdfRow {
    bold: bool,
    size: f64,
    segments: Vec<(f64, String)>,
}

fn pdf_row(bold: bool, columns: [f64; 4], cells: [String; 4]) -> PdfRow {
    PdfRow {
        bold,
        size: 10.0,
        segments: columns.into_iter().zip(cells).collect(),
    }
}

/// Baris tunggal rata kiri pada margin.
fn pdf_line(text: String, bold: bool) -> PdfRow {
    PdfRow {
        bold,
        size: if bold { 13.0 } else { 10.0 },
        segments: vec![(PDF_MARGIN, text)],
    }
}

/// Baris aturan (===== / _____): 12 pt agar memenuhi lebar halaman,
/// seperti contoh asli (69 karakter `=` = 467 pt penuh).
fn pdf_rule(text: String) -> PdfRow {
    PdfRow {
        bold: false,
        size: 12.0,
        segments: vec![(PDF_MARGIN, text)],
    }
}

/// Judul di tengah halaman, meniru posisi "Analysis of the food record" /
/// "Result" pada contoh (lebar karakter Times ~0.5 × ukuran font).
fn pdf_centered(text: &str) -> PdfRow {
    let size = 13.0;
    let width = text.chars().count() as f64 * size * 0.5;
    PdfRow {
        bold: true,
        size,
        segments: vec![(((595.4 - width) / 2.0).max(PDF_MARGIN), text.to_string())],
    }
}

/// Renderer PDF: tata letak A4 meniru contoh asli (Times New Roman, margin
/// 56,6 pt, leading 13,8 pt, tabel makanan halaman 1, HASIL PERHITUNGAN di
/// halaman baru dengan kolom rapat).
pub fn render_pdf(request: ExportRequest) -> Result<Vec<u8>, AppError> {
    let report = build_report(&request);
    let targets = request.targets;
    let separator = "=".repeat(69);
    // 78 karakter `_` pada 12 pt = 468 pt, memenuhi lebar halaman seperti contoh.
    let dash_line = "_".repeat(78);

    // ---- Halaman: catatan makanan ----
    let mut lines: Vec<PdfRow> = vec![
        pdf_centered("HASIL PERHITUNGAN DIET"),
        pdf_rule(separator.clone()),
        pdf_row(
            false,
            PDF_FOOD_COLUMNS,
            [
                "Nama Makanan".into(),
                "Jumlah".into(),
                "energy".into(),
                "carbohydr.".into(),
            ],
        ),
        pdf_rule(dash_line.clone()),
        pdf_line(String::new(), false),
    ];
    for meal in &report.meals {
        lines.push(pdf_line(meal.label.clone(), true));
        for food in &meal.foods {
            lines.push(pdf_row(
                false,
                PDF_FOOD_COLUMNS,
                [
                    pdf_truncate(&food.name, 42),
                    pdf_truncate(&food.amount, 12),
                    pdf_truncate(&format!("{} kcal", format_number(food.energy, 1, 0)), 12),
                    pdf_truncate(&format!("{} g", format_number(food.carbs, 1, 0)), 12),
                ],
            ));
        }
        let energy_pct = if report.total_energy > 0.0 {
            meal.energy / report.total_energy * 100.0
        } else {
            0.0
        };
        let carbs_pct = if report.total_carbs > 0.0 {
            meal.carbs / report.total_carbs * 100.0
        } else {
            0.0
        };
        lines.push(pdf_line(String::new(), false));
        lines.push(pdf_line(
            format!(
                "Meal analysis:  energy {} kcal ({} %),  carbohydrate {} g ({} %)",
                format_number(meal.energy, 1, 0),
                format_number(energy_pct, 0, 0),
                format_number(meal.carbs, 1, 0),
                format_number(carbs_pct, 0, 0)
            ),
            false,
        ));
        // Jarak antar waktu makan seperti contoh: 3 baris kosong.
        lines.push(pdf_line(String::new(), false));
        lines.push(pdf_line(String::new(), false));
        lines.push(pdf_line(String::new(), false));
    }

    // ---- Halaman baru: HASIL PERHITUNGAN (hitungan diet) ----
    let mut result: Vec<PdfRow> = vec![
        pdf_centered("HASIL PERHITUNGAN"),
        pdf_rule(separator.clone()),
        pdf_row(
            false,
            PDF_NUTRIENT_COLUMNS,
            [
                "Zat Gizi".into(),
                "hasil analisis".into(),
                "rekomendasi".into(),
                "persentase".into(),
            ],
        ),
        pdf_row(
            false,
            PDF_NUTRIENT_COLUMNS,
            [
                "content".into(),
                "nilai".into(),
                "nilai/hari".into(),
                "pemenuhan".into(),
            ],
        ),
        pdf_rule(dash_line.clone()),
    ];
    for (name, analysis, recommendation, percent) in nutrient_table_rows(&report, targets.as_ref())
    {
        result.push(pdf_row(
            false,
            PDF_NUTRIENT_COLUMNS,
            [
                pdf_truncate(&name, 26),
                pdf_truncate(&analysis, 20),
                pdf_truncate(&recommendation, 20),
                pdf_truncate(&percent, 12),
            ],
        ));
    }

    let food_page_capacity = PDF_PAGE_LINES;
    let mut pages: Vec<Vec<&PdfRow>> = vec![Vec::new()];
    for line in &lines {
        if pages.last().unwrap().len() >= food_page_capacity {
            pages.push(Vec::new());
        }
        pages.last_mut().unwrap().push(line);
    }
    // HASIL PERHITUNGAN selalu mulai di halaman baru, seperti contoh.
    pages.push(Vec::new());
    for line in &result {
        if pages.last().unwrap().len() >= food_page_capacity {
            pages.push(Vec::new());
        }
        pages.last_mut().unwrap().push(line);
    }

    let mut content_streams: Vec<String> = Vec::new();
    for page in &pages {
        let mut stream = String::from("BT\n");
        let mut y = PDF_TOP;
        for row in page {
            let font = if row.bold { "/F2" } else { "/F1" };
            for (x, text) in &row.segments {
                stream.push_str(&format!(
                    "{} {} Tf 1 0 0 1 {} {} Tm ({}) Tj\n",
                    font,
                    row.size,
                    x,
                    y,
                    pdf_escape_text(text)
                ));
            }
            y -= PDF_LEADING;
        }
        stream.push_str("ET");
        content_streams.push(stream);
    }

    let page_count = pages.len();
    let mut pdf = String::from("%PDF-1.4\n%\u{e2}\u{e3}\u{cf}\u{d3}\n");
    let mut offsets: Vec<usize> = Vec::new();
    let add_object = |pdf: &mut String, offsets: &mut Vec<usize>, body: &str| {
        offsets.push(pdf.len());
        pdf.push_str(&format!("{} 0 obj\n{}\nendobj\n", offsets.len(), body));
    };
    add_object(&mut pdf, &mut offsets, "<< /Type /Catalog /Pages 2 0 R >>");
    let kids: Vec<String> = (0..page_count)
        .map(|index| format!("{} 0 R", 6 + index * 2))
        .collect();
    add_object(
        &mut pdf,
        &mut offsets,
        &format!(
            "<< /Type /Pages /Kids [{}] /Count {page_count} >>",
            kids.join(" ")
        ),
    );
    add_object(
        &mut pdf,
        &mut offsets,
        "<< /Type /Font /Subtype /Type1 /BaseFont /Times-Roman /Encoding /WinAnsiEncoding >>",
    );
    add_object(
        &mut pdf,
        &mut offsets,
        "<< /Type /Font /Subtype /Type1 /BaseFont /Times-Bold /Encoding /WinAnsiEncoding >>",
    );
    for (index, stream) in content_streams.iter().enumerate() {
        let content_object = 5 + index * 2;
        let stream_length = stream.len();
        add_object(
            &mut pdf,
            &mut offsets,
            &format!("<< /Length {stream_length} >>\nstream\n{stream}\nendstream"),
        );
        add_object(
            &mut pdf,
            &mut offsets,
            &format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595.4 841.9] \
/Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {content_object} 0 R >>"
            ),
        );
    }
    let xref_offset = pdf.len();
    pdf.push_str(&format!("xref\n0 {}\n", offsets.len() + 1));
    pdf.push_str("0000000000 65535 f \n");
    for offset in &offsets {
        pdf.push_str(&format!("{offset:010} 00000 n \n"));
    }
    pdf.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF",
        offsets.len() + 1
    ));
    Ok(pdf.into_bytes())
}

// ---------------------------------------------------------------------------
// Writer ZIP ber-metode STORE (untuk DOCX) tanpa dependensi eksternal.
// ---------------------------------------------------------------------------

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// Tulis ZIP tanpa kompresi (STORE) berisi pasangan (nama berkas, isi).
pub fn zip_store(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut central: Vec<u8> = Vec::new();
    let mut offsets: Vec<u32> = Vec::new();
    for (name, data) in entries {
        let name_bytes = name.as_bytes();
        let crc = crc32(data);
        offsets.push(out.len() as u32);
        out.extend_from_slice(&0x0403_4b50_u32.to_le_bytes());
        out.extend_from_slice(&20_u16.to_le_bytes()); // version needed
        out.extend_from_slice(&0_u16.to_le_bytes()); // flags
        out.extend_from_slice(&0_u16.to_le_bytes()); // method STORE
        out.extend_from_slice(&0_u16.to_le_bytes()); // mod time
        out.extend_from_slice(&0_u16.to_le_bytes()); // mod date
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes()); // extra length
        out.extend_from_slice(name_bytes);
        out.extend_from_slice(data);
        central.extend_from_slice(&0x0201_4b50_u32.to_le_bytes());
        central.extend_from_slice(&20_u16.to_le_bytes()); // version made by
        central.extend_from_slice(&20_u16.to_le_bytes()); // version needed
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes()); // extra
        central.extend_from_slice(&0_u16.to_le_bytes()); // comment
        central.extend_from_slice(&0_u16.to_le_bytes()); // disk number
        central.extend_from_slice(&0_u16.to_le_bytes()); // internal attrs
        central.extend_from_slice(&0_u32.to_le_bytes()); // external attrs
        central.extend_from_slice(&offsets.last().copied().unwrap_or(0).to_le_bytes());
        central.extend_from_slice(name_bytes);
    }
    let central_offset = out.len() as u32;
    out.extend_from_slice(&central);
    let central_size = central.len() as u32;
    out.extend_from_slice(&0x0605_4b50_u32.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes()); // disk
    out.extend_from_slice(&0_u16.to_le_bytes()); // start disk
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&central_size.to_le_bytes());
    out.extend_from_slice(&central_offset.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes()); // comment length
    out
}

/// Utilitas pengujian: baca ZIP ber-metode STORE hasil `zip_store` untuk
/// memverifikasi struktur dan CRC tanpa menambah dependensi pada crate tests.
pub mod test_support {
    /// Kembalikan (entri, offset central directory) dari ZIP STORE.
    pub fn parse_stored_zip(bytes: &[u8]) -> (Vec<(String, Vec<u8>)>, usize) {
        let read_u16 =
            |offset: usize| u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap());
        let read_u32 =
            |offset: usize| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let mut entries = Vec::new();
        let mut cursor = 0;
        while cursor + 4 <= bytes.len() && read_u32(cursor) == 0x0403_4b50 {
            let name_length = read_u16(cursor + 26) as usize;
            let extra_length = read_u16(cursor + 28) as usize;
            let crc = read_u32(cursor + 14);
            let compressed_size = read_u32(cursor + 18) as usize;
            let data_start = cursor + 30 + name_length + extra_length;
            let name =
                String::from_utf8_lossy(&bytes[cursor + 30..cursor + 30 + name_length]).to_string();
            let data = bytes[data_start..data_start + compressed_size].to_vec();
            assert_eq!(
                super::crc32(&data),
                crc,
                "CRC tidak cocok untuk entri {name}"
            );
            entries.push((name, data));
            cursor = data_start + compressed_size;
        }
        let central_offset = read_u32(bytes.len() - 22 - 6) as usize;
        (entries, central_offset)
    }
}

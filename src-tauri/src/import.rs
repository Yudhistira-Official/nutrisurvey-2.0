use crate::{error::AppError, storage::Storage};
use csv::ReaderBuilder;
use sqlx::{Sqlite, Transaction};
use std::collections::HashMap;

pub async fn import_csv(
    storage: &Storage,
    bytes: &[u8],
    source_name: &str,
) -> Result<u64, AppError> {
    if bytes.is_empty() {
        return Ok(0);
    }
    let delimiter = detect_delimiter(bytes);
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(bytes);
    let raw_headers = reader.headers().map_err(import_error)?.clone();
    let headers: Vec<String> = raw_headers.iter().map(normalize_header).collect();
    if headers.is_empty() {
        return Ok(0);
    }
    let scraper = headers
        .iter()
        .any(|header| header.starts_with("komponen_nutrient_"))
        && headers.iter().any(|header| header == "makanan");
    let mut transaction = storage.transaction().await?;
    let result = if scraper {
        import_scraper(&mut transaction, &headers, &mut reader).await
    } else {
        import_standard(&mut transaction, &headers, &mut reader).await
    };
    match result {
        Ok(count) => {
            transaction.commit().await?;
            Ok(count)
        }
        Err(error) => Err(AppError::Import(format!("{source_name}: {error}"))),
    }
}

async fn import_standard(
    tx: &mut Transaction<'_, Sqlite>,
    headers: &[String],
    reader: &mut csv::Reader<&[u8]>,
) -> Result<u64, String> {
    let food_index = find_header(headers, &["nama makanan", "foodname"])
        .ok_or_else(|| "Header 'Nama Makanan' tidak ditemukan.".to_string())?;
    let category_index = find_header(headers, &["kategori", "category"]);
    let serving_count_index = find_header(headers, &["jumlah sajian"]);
    let serving_index = find_header(headers, &["per sajian"]);
    let mut nutrients = nutrient_map(tx).await?;
    let mut count = 0;
    for record in reader.records() {
        let record = record.map_err(|error| error.to_string())?;
        let Some(name) = field(&record, food_index).filter(|value| !value.trim().is_empty()) else {
            continue;
        };
        let category = category_index.and_then(|index| field(&record, index));
        let servings = serving_count_index
            .and_then(|index| field(&record, index))
            .and_then(|value| parse_number(&value));
        let serving = serving_index
            .and_then(|index| field(&record, index))
            .and_then(|value| parse_serving(&value));
        let food_id = insert_food(tx, &name, category.as_deref(), serving, servings).await?;
        for (index, header) in headers.iter().enumerate() {
            if index == food_index || Some(index) == category_index || is_metadata(header) {
                continue;
            }
            let Some(value) = field(&record, index) else {
                continue;
            };
            if let Some((amount, unit)) = parse_nutrient(&value) {
                let nutrient_id = nutrient_id(tx, &mut nutrients, header, unit.as_deref()).await?;
                sqlx::query("INSERT OR IGNORE INTO food_nutrients (food_id, nutrient_id, amount) VALUES (?, ?, ?)")
                    .bind(food_id).bind(nutrient_id).bind(amount).execute(&mut **tx).await.map_err(|e| e.to_string())?;
            }
        }
        count += 1;
    }
    Ok(count)
}

async fn import_scraper(
    tx: &mut Transaction<'_, Sqlite>,
    headers: &[String],
    reader: &mut csv::Reader<&[u8]>,
) -> Result<u64, String> {
    let food_index = headers
        .iter()
        .position(|header| header == "makanan")
        .unwrap();
    let category_index = headers.iter().position(|header| header == "kategori");
    let mut nutrients = nutrient_map(tx).await?;
    let mut count = 0;
    for record in reader.records() {
        let record = record.map_err(|error| error.to_string())?;
        let Some(raw_name) = field(&record, food_index).filter(|value| !value.trim().is_empty())
        else {
            continue;
        };
        let name = raw_name
            .split(';')
            .next()
            .unwrap_or(&raw_name)
            .trim()
            .to_string();
        let category = category_index.and_then(|index| field(&record, index));
        let food_id = match sqlx::query_scalar::<_, i64>("SELECT id FROM foods WHERE name = ?")
            .bind(&name)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| e.to_string())?
        {
            Some(id) => id,
            None => insert_food(tx, &name, category.as_deref(), None, None).await?,
        };
        for index in 1..=48 {
            let name_index = headers
                .iter()
                .position(|header| header == &format!("komponen_nutrient_{index}"));
            let value_index = headers
                .iter()
                .position(|header| header == &format!("isi_nutrient_{index}"));
            let (Some(name_index), Some(value_index)) = (name_index, value_index) else {
                continue;
            };
            let (Some(nutrient_name), Some(value)) =
                (field(&record, name_index), field(&record, value_index))
            else {
                continue;
            };
            if let Some((amount, unit)) = parse_nutrient(&value) {
                let nutrient_id =
                    nutrient_id(tx, &mut nutrients, &nutrient_name, unit.as_deref()).await?;
                sqlx::query("INSERT INTO food_nutrients (food_id, nutrient_id, amount) VALUES (?, ?, ?) ON CONFLICT(food_id, nutrient_id) DO UPDATE SET amount = excluded.amount")
                    .bind(food_id).bind(nutrient_id).bind(amount).execute(&mut **tx).await.map_err(|e| e.to_string())?;
            }
        }
        count += 1;
    }
    Ok(count)
}

async fn insert_food(
    tx: &mut Transaction<'_, Sqlite>,
    name: &str,
    category: Option<&str>,
    serving: Option<(f64, String)>,
    servings: Option<f64>,
) -> Result<i64, String> {
    let (size, unit) = serving.unwrap_or((100.0, "g".into()));
    Ok(sqlx::query("INSERT INTO foods (name, normalized_name, category, serving_size, serving_unit, servings_per_container) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(name).bind(name.to_lowercase()).bind(category).bind(size).bind(unit).bind(servings.unwrap_or(1.0)).execute(&mut **tx).await.map_err(|e| e.to_string())?.last_insert_rowid())
}

async fn nutrient_map(tx: &mut Transaction<'_, Sqlite>) -> Result<HashMap<String, i64>, String> {
    Ok(
        sqlx::query_as::<_, (String, i64)>("SELECT normalized_name, id FROM nutrients")
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .collect(),
    )
}

async fn nutrient_id(
    tx: &mut Transaction<'_, Sqlite>,
    nutrients: &mut HashMap<String, i64>,
    name: &str,
    unit: Option<&str>,
) -> Result<i64, String> {
    let key = name.to_lowercase();
    if let Some(id) = nutrients.get(&key) {
        return Ok(*id);
    }
    let id = sqlx::query("INSERT INTO nutrients (name, normalized_name, unit) VALUES (?, ?, ?)")
        .bind(name)
        .bind(&key)
        .bind(unit.unwrap_or("mg"))
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?
        .last_insert_rowid();
    nutrients.insert(key, id);
    Ok(id)
}

fn detect_delimiter(bytes: &[u8]) -> u8 {
    let semicolons = bytes
        .iter()
        .take(4096)
        .filter(|byte| **byte == b';')
        .count();
    let commas = bytes
        .iter()
        .take(4096)
        .filter(|byte| **byte == b',')
        .count();
    if semicolons >= commas {
        b';'
    } else {
        b','
    }
}
fn normalize_header(value: &str) -> String {
    value.trim().trim_matches('\u{feff}').to_lowercase()
}
fn find_header(headers: &[String], names: &[&str]) -> Option<usize> {
    headers
        .iter()
        .position(|header| names.contains(&header.as_str()))
}
fn field(record: &csv::StringRecord, index: usize) -> Option<String> {
    record.get(index).map(str::trim).map(str::to_string)
}
fn is_metadata(header: &str) -> bool {
    matches!(header, "id" | "jumlah sajian" | "per sajian") || header.contains("web_scraper")
}
fn parse_number(value: &str) -> Option<f64> {
    value.trim().replace(',', ".").parse().ok()
}
fn parse_serving(value: &str) -> Option<(f64, String)> {
    let value = value.replace(',', ".");
    let mut number = String::new();
    let mut unit = String::new();
    for character in value.chars() {
        if character.is_ascii_digit() || character == '.' {
            number.push(character);
        } else if character.is_ascii_alphabetic() {
            unit.push(character);
        }
    }
    Some((number.parse().ok()?, unit)).filter(|(_, unit)| !unit.is_empty())
}
fn parse_nutrient(value: &str) -> Option<(f64, Option<String>)> {
    let value = value.trim();
    if value.is_empty() || value == "-" || value == "0" {
        return None;
    }
    let normalized = value.replace(',', ".");
    let mut parts = normalized.split_whitespace();
    let amount = parts.next()?.parse().ok()?;
    let unit = parts.next().map(str::to_string).or_else(|| {
        value
            .chars()
            .skip_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect::<String>()
            .into()
    });
    Some((amount, unit))
}
fn import_error(error: csv::Error) -> AppError {
    AppError::Import(error.to_string())
}

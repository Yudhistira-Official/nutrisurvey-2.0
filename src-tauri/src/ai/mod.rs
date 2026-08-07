use crate::{
    error::AppError,
    meals,
    models::{AiRequest, MappedMealItem},
    storage::Storage,
};
use reqwest::{redirect::Policy, Client, Url};
use serde::Serialize;
use serde_json::Value;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

mod anthropic;
mod google;
pub use google::{google_endpoint, google_endpoint_for_client};
mod openai;
pub mod prompt;

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use tokio::sync::Semaphore;

static CANCELLED_REQUESTS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static AI_REQUEST_GATE: OnceLock<Semaphore> = OnceLock::new();

fn ai_request_gate() -> &'static Semaphore {
    AI_REQUEST_GATE.get_or_init(|| Semaphore::new(1))
}

fn cancelled_requests() -> &'static Mutex<HashSet<String>> {
    CANCELLED_REQUESTS.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn cancel_request(request_id: &str) {
    if let Ok(mut requests) = cancelled_requests().lock() {
        requests.insert(request_id.to_owned());
    }
}

pub(crate) fn is_cancelled(request_id: &str) -> bool {
    cancelled_requests()
        .lock()
        .map(|requests| requests.contains(request_id))
        .unwrap_or(false)
}

const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);

pub async fn generate_menu(
    storage: &Storage,
    request: AiRequest,
) -> Result<Vec<MappedMealItem>, AppError> {
    let _request_permit = ai_request_gate()
        .acquire()
        .await
        .map_err(|_| AppError::Ai("AI request gate is unavailable".into()))?;
    validate_request(&request)?;
    let request = with_candidate_catalog(storage, request).await?;
    let (base_url, socket) =
        resolve_and_pin(&request.base_url, || resolve_host(&request.base_url))?;
    let host = base_url
        .host_str()
        .ok_or_else(|| AppError::Validation("AI base URL is invalid".into()))?;
    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .redirect(Policy::none())
        .resolve(host, socket)
        .build()
        .map_err(|_| AppError::Ai("unable to configure AI client".into()))?;
    generate_menu_with_client(storage, request, &client).await
}

pub async fn stream_menu(
    storage: &Storage,
    request: AiRequest,
    channel: tauri::ipc::Channel<String>,
) -> Result<Vec<MappedMealItem>, AppError> {
    let _request_permit = ai_request_gate()
        .acquire()
        .await
        .map_err(|_| AppError::Ai("AI request gate is unavailable".into()))?;
    validate_request(&request)?;
    let request = with_candidate_catalog(storage, request).await?;
    let (base_url, socket) =
        resolve_and_pin(&request.base_url, || resolve_host(&request.base_url))?;
    let host = base_url
        .host_str()
        .ok_or_else(|| AppError::Validation("AI base URL is invalid".into()))?;
    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .redirect(Policy::none())
        .resolve(host, socket)
        .build()
        .map_err(|_| AppError::Ai("unable to configure AI client".into()))?;
    let mut current = request;
    let max_attempts = if current.verify_menu { 3 } else { 1 };
    let mut best: Option<(Vec<MappedMealItem>, f64)> = None;
    for attempt in 0..max_attempts {
        before_next_ai_phase(&current.request_id)?;
        let streaming_provider = matches!(
            current.provider.trim().to_ascii_lowercase().as_str(),
            "openai" | "openrouter" | "custom"
        );
        if attempt > 0 {
            let _ = channel.send(format!(
                "\n\n[Verifikasi {}/3: menghitung nutrisi database dan memperbaiki menu...]\n\n",
                attempt + 1
            ));
        }
        let content_result = if streaming_provider {
            openai::generate_stream(&client, &current, |token| {
                let _ = channel.send(token.to_owned());
            })
            .await
        } else {
            generate_content(&client, &current).await
        };
        let content = match content_result {
            Ok(content) => {
                if !streaming_provider {
                    send_stream_chunks(&channel, &content, &current.request_id).await;
                }
                before_next_ai_phase(&current.request_id)?;
                content
            }
            Err(error) if attempt + 1 < max_attempts && !is_rate_limited(&error) => {
                let _ = channel.send(format!(
                    "\n\n[Verifikasi {}/3: request AI gagal ({}). AI mengulangi...]\n\n",
                    attempt + 1,
                    error
                ));
                current.prompt = format!("{}\n\nRequest sebelumnya gagal. Kembalikan JSON valid saja, tanpa markdown atau prosa.", current.prompt);
                continue;
            }
            Err(error) => return Err(error),
        };
        let plan = match prompt::parse_meal_plan(&content) {
            Ok(plan) => plan,
            Err(error) if attempt + 1 < max_attempts => {
                let _ = channel.send(format!(
                    "\n\n[Verifikasi {}/3: respons AI tidak valid ({}). AI mengulangi...]\n\n",
                    attempt + 1,
                    error
                ));
                current.prompt = format!("{}\n\nRespons sebelumnya tidak valid JSON. Kembalikan JSON valid saja, tanpa markdown atau prosa.", current.prompt);
                continue;
            }
            Err(error) => return Err(error),
        };
        let mapped = meals::map_ai_items(storage, &plan).await?;
        let mapped_count = mapped.len();
        let menu =
            normalize_menu_to_tdee(merge_revision_menu(&current, mapped), current.target_tdee);
        let mut report = verify_menu_targets(
            &menu,
            current.target_tdee,
            current.target_carbs,
            current.target_protein,
            current.target_fat,
        );
        if mapped_count < plan.len() {
            report.is_valid = false;
            report.feedback = format!(
                "{}; {} dari {} item AI tidak ditemukan dalam database",
                report.feedback,
                plan.len() - mapped_count,
                plan.len()
            );
            report.error_score += (plan.len() - mapped_count) as f64;
        }
        if best
            .as_ref()
            .is_none_or(|(_, score)| report.error_score < *score)
        {
            best = Some((menu.clone(), report.error_score));
        }
        if report.is_valid || attempt + 1 == max_attempts {
            return Ok(menu);
        }
        before_next_ai_phase(&current.request_id)?;
        let _ = channel.send(format!(
            "\n\n[Verifikasi {}/3 belum sesuai: {}. AI memperbaiki item database yang bermasalah...]\n\n",
            attempt + 1,
            report.feedback
        ));
        current.active_menu = serde_json::to_string(&menu)
            .map_err(|_| AppError::Ai("AI verification context could not be serialized".into()))?;
        current.revision = true;
        current.prompt = format!("{}\n\nVERIFIKASI SISTEM (attempt {}/3): {}\nMenu aktif di atas adalah hasil pemetaan DATABASE SQLITE dan wajib dipertahankan. Jangan membuat ulang seluruh menu. Kembalikan hanya item yang perlu diperbaiki dalam format meal_plan, gunakan hanya makanan yang ada di katalog database, dan ubah suggested_grams atau food_keyword hanya pada item bermasalah. Semua item lain akan dipertahankan sistem.", current.prompt, attempt + 1, report.feedback);
    }
    Ok(best.map(|(menu, _)| menu).unwrap_or_default())
}

async fn with_candidate_catalog(
    storage: &Storage,
    mut request: AiRequest,
) -> Result<AiRequest, AppError> {
    if request.candidate_catalog.trim().is_empty() {
        let candidates = crate::foods::ai_candidate_catalog(storage, 300).await?;
        request.candidate_catalog = candidates
            .into_iter()
            .map(|food| {
                let energy = food.nutrients.get("energi").copied().unwrap_or(0.0);
                let carbs = food
                    .nutrients
                    .get("karbohidrat total")
                    .copied()
                    .unwrap_or(0.0);
                let protein = food.nutrients.get("protein").copied().unwrap_or(0.0);
                let fat = food.nutrients.get("lemak total").copied().unwrap_or(0.0);
                format!(
                    "{} | serving {:.0} {} | energi {:.1} | KH {:.1} | protein {:.1} | lemak {:.1}",
                    food.name, food.serving_size, food.serving_unit, energy, carbs, protein, fat
                )
            })
            .collect::<Vec<_>>()
            .join("\\n");
    }
    Ok(request)
}

async fn send_stream_chunks(
    channel: &tauri::ipc::Channel<String>,
    content: &str,
    request_id: &str,
) {
    let chars = content.chars().collect::<Vec<_>>();
    for chunk in chars.chunks(48) {
        if is_cancelled(request_id) {
            return;
        }
        let _ = channel.send(chunk.iter().collect());
        tokio::time::sleep(Duration::from_millis(24)).await;
    }
}

pub async fn generate_menu_with_client(
    storage: &Storage,
    request: AiRequest,
    client: &Client,
) -> Result<Vec<MappedMealItem>, AppError> {
    validate_request(&request)?;
    let mut best: Option<(Vec<MappedMealItem>, f64)> = None;
    let mut current = request;
    let max_attempts = if current.verify_menu { 3 } else { 1 };
    for attempt in 0..max_attempts {
        let content = match generate_content(client, &current).await {
            Ok(content) => content,
            Err(error) if is_rate_limited(&error) => return Err(error),
            Err(error) => return Err(error),
        };
        let plan = prompt::parse_meal_plan(&content)?;
        let mapped = meals::map_ai_items(storage, &plan).await?;
        let mapped_count = mapped.len();
        let menu =
            normalize_menu_to_tdee(merge_revision_menu(&current, mapped), current.target_tdee);
        let mut report = verify_menu_targets(
            &menu,
            current.target_tdee,
            current.target_carbs,
            current.target_protein,
            current.target_fat,
        );
        if mapped_count < plan.len() {
            report.is_valid = false;
            report.feedback = format!(
                "{}; {} dari {} item AI tidak ditemukan dalam database",
                report.feedback,
                plan.len() - mapped_count,
                plan.len()
            );
            report.error_score += (plan.len() - mapped_count) as f64;
        }
        let score = report.error_score;
        if best
            .as_ref()
            .is_none_or(|(_, best_score)| score < *best_score)
        {
            best = Some((menu.clone(), score));
        }
        if report.is_valid || attempt == 2 {
            return Ok(menu);
        }
        current.active_menu = serde_json::to_string(&menu)
            .map_err(|_| AppError::Ai("AI verification context could not be serialized".into()))?;
        current.revision = true;
        current.prompt = format!("{}\n\nVERIFIKASI SISTEM (attempt {}/3): {}\nMenu aktif di atas adalah hasil pemetaan DATABASE SQLITE dan wajib dipertahankan. Jangan membuat ulang seluruh menu. Kembalikan hanya item yang perlu diperbaiki dalam format meal_plan, gunakan hanya makanan yang ada di katalog database, dan ubah suggested_grams atau food_keyword hanya pada item bermasalah. Semua item lain akan dipertahankan sistem.", current.prompt, attempt + 1, report.feedback);
    }
    Ok(best.map(|(menu, _)| menu).unwrap_or_default())
}

fn before_next_ai_phase(request_id: &str) -> Result<(), AppError> {
    if is_cancelled(request_id) {
        Err(AppError::Ai("AI request cancelled".into()))
    } else {
        Ok(())
    }
}

fn is_rate_limited(error: &AppError) -> bool {
    error.to_string().contains("HTTP 429")
        || error.to_string().contains("rate limit")
        || error.to_string().contains("Rate limit")
        || error.to_string().contains("FreeUsageLimitError")
}

async fn generate_content(client: &Client, request: &AiRequest) -> Result<String, AppError> {
    match request.provider.trim().to_ascii_lowercase().as_str() {
        "google" | "gemini" => google::generate(client, request).await,
        "anthropic" | "claude" => anthropic::generate(client, request).await,
        "openai" | "openrouter" | "custom" => openai::generate(client, request).await,
        _ => Err(AppError::Validation("unsupported AI provider".into())),
    }
}

fn merge_revision_menu(
    request: &AiRequest,
    mut mapped: Vec<MappedMealItem>,
) -> Vec<MappedMealItem> {
    if !request.revision || explicit_delete_requested(&request.prompt) {
        return mapped;
    }
    let Ok(active) = serde_json::from_str::<Vec<MappedMealItem>>(&request.active_menu) else {
        return mapped;
    };
    for previous in active {
        let exists = mapped.iter().any(|item| {
            item.matched_food_id == previous.matched_food_id && item.meal_type == previous.meal_type
        });
        if !exists {
            mapped.push(previous);
        }
    }
    mapped
}

fn explicit_delete_requested(prompt: &str) -> bool {
    let value = prompt.to_ascii_lowercase();
    [
        "hapus",
        "menghapus",
        "hilangkan",
        "buang",
        "remove",
        "delete",
    ]
    .iter()
    .any(|word| value.contains(word))
}

#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub is_valid: bool,
    pub feedback: String,
    pub error_score: f64,
}

pub fn verify_menu_targets(
    menu: &[MappedMealItem],
    target_tdee: i32,
    target_carbs: i32,
    target_protein: i32,
    target_fat: i32,
) -> VerificationReport {
    let totals = menu
        .iter()
        .fold((0.0, 0.0, 0.0, 0.0), |(kcal, carbs, protein, fat), item| {
            (
                kcal + item.calories,
                carbs + item.carbohydrate,
                protein + item.protein,
                fat + item.fat,
            )
        });
    let targets = [
        ("Energi", totals.0, target_tdee),
        ("Karbohidrat", totals.1, target_carbs),
        ("Protein", totals.2, target_protein),
        ("Lemak", totals.3, target_fat),
    ];
    let mut feedback = Vec::new();
    let mut error_score = 0.0;
    for (name, actual, target) in targets {
        let difference = (actual - target as f64).abs();
        let tolerance = (target as f64 * 0.15).max(5.0);
        error_score += difference / (target as f64).max(1.0);
        if difference > tolerance {
            feedback.push(format!(
                "{name} aktual {:.1}, target {}, selisih {:.1}",
                actual,
                target,
                actual - target as f64
            ));
        }
    }
    VerificationReport {
        is_valid: !menu.is_empty() && feedback.is_empty(),
        feedback: if feedback.is_empty() {
            "Semua target terpenuhi.".into()
        } else {
            feedback.join("; ")
        },
        error_score,
    }
}

fn normalize_menu_to_tdee(mut menu: Vec<MappedMealItem>, target_tdee: i32) -> Vec<MappedMealItem> {
    if menu.is_empty() || target_tdee <= 0 {
        return menu;
    }
    let calories = menu.iter().map(|item| item.calories).sum::<f64>();
    if calories <= 0.0 || (calories - target_tdee as f64).abs() <= target_tdee as f64 * 0.05 {
        return menu;
    }
    let factor = target_tdee as f64 / calories;
    menu.retain_mut(|item| {
        if item.suggested_grams <= 0 {
            return false;
        }
        let grams = (item.suggested_grams as f64 * factor).round() as i32;
        if grams < 25 {
            return false;
        }
        let nutrient_factor = grams as f64 / item.suggested_grams as f64;
        item.suggested_grams = grams;
        item.calories = round(item.calories * nutrient_factor);
        item.protein = round(item.protein * nutrient_factor);
        item.fat = round(item.fat * nutrient_factor);
        item.carbohydrate = round(item.carbohydrate * nutrient_factor);
        for amount in item.nutrients.values_mut() {
            *amount = round(*amount * nutrient_factor);
        }
        true
    });
    menu
}

fn round(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn validate_request(request: &AiRequest) -> Result<(), AppError> {
    if request.target_tdee <= 0
        || request.model.trim().is_empty()
        || request.available_meal_types.is_empty()
    {
        return Err(AppError::Validation(
            "AI model and meal types are required".into(),
        ));
    }
    if !request.base_url.trim().is_empty() {
        endpoint_shape(&request.base_url)?;
    }
    Ok(())
}

fn endpoint_shape(base_url: &str) -> Result<Url, AppError> {
    let url = Url::parse(base_url.trim())
        .map_err(|_| AppError::Validation("AI base URL is invalid".into()))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || (!is_loopback_host(&url) && is_private_host(&url))
    {
        return Err(AppError::Validation("AI base URL is invalid".into()));
    }
    Ok(url)
}

pub fn endpoint_for_client(base_url: &str, suffix: &str) -> Result<Url, AppError> {
    join_endpoint(endpoint_shape(base_url)?, suffix)
}

fn resolve_host(base_url: &str) -> Result<Vec<IpAddr>, std::io::Error> {
    let parsed = Url::parse(base_url.trim())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid URL"))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing host"))?
        .to_owned();
    let port = parsed
        .port_or_known_default()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing port"))?;
    std::net::ToSocketAddrs::to_socket_addrs(&(host.as_str(), port))
        .map(|addresses| addresses.map(|address| address.ip()).collect())
}

pub fn resolve_and_pin<F, E>(base_url: &str, resolver: F) -> Result<(Url, SocketAddr), AppError>
where
    F: FnOnce() -> Result<Vec<IpAddr>, E>,
{
    let url = endpoint_shape(base_url)?;
    let port = url
        .port_or_known_default()
        .ok_or_else(|| AppError::Validation("AI base URL is invalid".into()))?;
    let addresses =
        resolver().map_err(|_| AppError::Validation("AI base URL could not be resolved".into()))?;
    validate_resolved_addresses(&url, &addresses)?;
    let address = if is_loopback_host(&url) {
        addresses
            .iter()
            .find(|address| address.is_ipv4())
            .copied()
            .or_else(|| addresses.first().copied())
    } else {
        addresses.first().copied()
    }
    .ok_or_else(|| AppError::Validation("AI base URL could not be resolved".into()))?;
    Ok((url, SocketAddr::new(address, port)))
}

pub fn endpoint(base_url: &str, suffix: &str) -> Result<Url, AppError> {
    endpoint_with_resolver(base_url, suffix, || {
        let parsed = Url::parse(base_url.trim())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid URL"))?;
        let host = parsed
            .host_str()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing host"))?
            .to_owned();
        let port = parsed
            .port_or_known_default()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing port"))?;
        std::net::ToSocketAddrs::to_socket_addrs(&(host.as_str(), port))
            .map(|addresses| addresses.map(|address| address.ip()).collect())
    })
}

pub fn endpoint_with_resolver<F, E>(
    base_url: &str,
    suffix: &str,
    resolver: F,
) -> Result<Url, AppError>
where
    F: FnOnce() -> Result<Vec<IpAddr>, E>,
{
    let value = base_url.trim();
    let url =
        Url::parse(value).map_err(|_| AppError::Validation("AI base URL is invalid".into()))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || (!is_loopback_host(&url) && is_private_host(&url))
    {
        return Err(AppError::Validation("AI base URL is invalid".into()));
    }
    let addresses =
        resolver().map_err(|_| AppError::Validation("AI base URL could not be resolved".into()))?;
    validate_resolved_addresses(&url, &addresses)?;
    join_endpoint(url, suffix)
}

fn join_endpoint(mut url: Url, suffix: &str) -> Result<Url, AppError> {
    let suffix = suffix.trim_matches('/');
    if suffix.is_empty() {
        return Ok(url);
    }
    let existing = url.path().trim_end_matches('/');
    if existing.ends_with(&format!("/{suffix}")) || existing == suffix {
        return Ok(url);
    }
    let base_path = url.path().trim_end_matches('/');
    url.set_path(&format!("{base_path}/{suffix}"));
    Ok(url)
}

fn is_private_ip(address: &IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => is_private_ipv4(*address),
        IpAddr::V6(address) => is_private_ipv6(*address),
    }
}

fn validate_resolved_addresses(url: &Url, addresses: &[IpAddr]) -> Result<(), AppError> {
    let loopback_host = is_loopback_host(url);
    if addresses.is_empty()
        || addresses.iter().any(|address| {
            if loopback_host {
                !address.is_loopback()
            } else {
                is_private_ip(address)
            }
        })
    {
        return Err(AppError::Validation(
            "AI base URL resolves to an unsafe address".into(),
        ));
    }
    Ok(())
}

fn is_loopback_host(url: &Url) -> bool {
    url.host_str().is_some_and(|host| {
        let normalized = host.trim_start_matches('[').trim_end_matches(']');
        normalized.eq_ignore_ascii_case("localhost") || normalized == "127.0.0.1"
    })
}

fn is_private_host(url: &Url) -> bool {
    let Some(host) = url.host_str() else {
        return true;
    };
    let host = host.trim_start_matches('[').trim_end_matches(']');
    if let Ok(address) = host.parse::<IpAddr>() {
        return match address {
            IpAddr::V4(address) => is_private_ipv4(address),
            IpAddr::V6(address) => is_private_ipv6(address),
        };
    }
    let domain = host.trim_end_matches('.').to_ascii_lowercase();
    domain == "localhost" || domain.ends_with(".localhost") || domain.ends_with(".local")
}

fn is_private_ipv4(address: Ipv4Addr) -> bool {
    address.is_private()
        || address.is_loopback()
        || address.is_link_local()
        || address.is_unspecified()
        || address.octets()[0] == 100 && (64..=127).contains(&address.octets()[1])
        || address.octets()[0] == 192 && address.octets()[1] == 0 && address.octets()[2] == 0
        || address.octets()[0] == 198 && (18..=19).contains(&address.octets()[1])
        || address.octets()[0] >= 224
}

fn is_private_ipv6(address: Ipv6Addr) -> bool {
    address.to_ipv4_mapped().is_some_and(is_private_ipv4)
        || address.is_loopback()
        || address.is_unspecified()
        || (address.segments()[0] & 0xfe00) == 0xfc00
        || (address.segments()[0] & 0xffc0) == 0xfe80
}

pub fn request_error(status: reqwest::StatusCode, body: &str, secret: &str) -> AppError {
    let detail = provider_error_detail(body, secret);
    if detail.is_empty() {
        AppError::Ai(format!("AI provider returned HTTP {}", status.as_u16()))
    } else {
        AppError::Ai(format!(
            "AI provider returned HTTP {}: {}",
            status.as_u16(),
            detail
        ))
    }
}

fn sanitize_error(value: &str) -> String {
    value
        .replace("Bearer ", "Bearer [REDACTED]")
        .chars()
        .take(300)
        .collect()
}

fn provider_error_detail(body: &str, secret: &str) -> String {
    let detail = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| {
            ["/error/message", "/error", "/message", "/detail"]
                .iter()
                .find_map(|path| {
                    value
                        .pointer(path)
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                })
        })
        .unwrap_or_else(|| body.trim().replace(['\n', '\r'], " "));
    let sanitized = detail
        .replace("Bearer ", "Bearer [REDACTED]")
        .replace(secret, "[REDACTED]");
    sanitized.chars().take(300).collect()
}

pub fn parse_any_response(body: &str, fields: &[&str]) -> Result<String, AppError> {
    for field in fields {
        if let Ok(value) = parse_response(body, field) {
            return Ok(value);
        }
    }
    Err(AppError::Ai(
        "AI provider response is missing content".into(),
    ))
}

pub(crate) fn parse_response(body: &str, field: &str) -> Result<String, AppError> {
    let value: Value = serde_json::from_str(body)
        .map_err(|_| AppError::Ai("AI provider returned malformed JSON".into()))?;
    extract_text_value(value.pointer(field))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| AppError::Ai("AI provider response is missing content".into()))
}

fn extract_text_value(value: Option<&Value>) -> Option<String> {
    let value = value?;
    if let Some(text) = value.as_str() {
        return Some(text.to_owned());
    }
    if let Some(items) = value.as_array() {
        let text = items
            .iter()
            .filter_map(|item| {
                item.as_str()
                    .map(str::to_owned)
                    .or_else(|| item.get("text").and_then(Value::as_str).map(str::to_owned))
            })
            .collect::<Vec<_>>()
            .join("");
        if !text.is_empty() {
            return Some(text);
        }
    }
    for key in ["text", "content", "parts", "output_text"] {
        if let Some(text) = extract_text_value(value.get(key)) {
            return Some(text);
        }
    }
    None
}

pub(crate) async fn send_json<T: Serialize>(
    _client: &Client,
    request: reqwest::RequestBuilder,
    payload: &T,
    secret: &str,
) -> Result<String, AppError> {
    let response = request
        .header(reqwest::header::ACCEPT_ENCODING, "identity")
        .json(payload)
        .send()
        .await
        .map_err(|error| {
            AppError::Ai(format!(
                "AI provider request failed: {}",
                sanitize_error(&error.to_string())
            ))
        })?;
    let status = response.status();
    let body = String::from_utf8_lossy(&response.bytes().await.map_err(|error| {
        AppError::Ai(format!(
            "AI provider response could not be read: {}",
            sanitize_error(&error.to_string())
        ))
    })?)
    .into_owned();
    if !status.is_success() {
        return Err(request_error(status, &body, secret));
    }
    Ok(body)
}

#[derive(Debug, Serialize)]
pub(crate) struct OpenAiPayload<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    temperature: f64,
}

#[derive(Debug, Serialize)]
pub(crate) struct Message<'a> {
    role: &'a str,
    content: String,
}

pub(crate) fn payload(request: &AiRequest) -> OpenAiPayload<'_> {
    OpenAiPayload {
        model: &request.model,
        messages: vec![
            Message {
                role: "system",
                content:
                    "You are a nutrition meal-planning assistant. Always return valid JSON only."
                        .into(),
            },
            Message {
                role: "user",
                content: prompt::build(request),
            },
        ],
        temperature: 0.2,
    }
}

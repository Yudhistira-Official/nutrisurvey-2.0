use crate::{
    error::AppError,
    meals,
    models::{AiRequest, MappedMealItem},
    storage::Storage,
};
use reqwest::{Client, Url};
use serde::Serialize;
use serde_json::Value;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

mod anthropic;
mod google;
pub use google::google_endpoint;
mod openai;
pub mod prompt;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn generate_menu(
    storage: &Storage,
    request: AiRequest,
) -> Result<Vec<MappedMealItem>, AppError> {
    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|_| AppError::Ai("unable to configure AI client".into()))?;
    generate_menu_with_client(storage, request, &client).await
}

pub async fn generate_menu_with_client(
    storage: &Storage,
    request: AiRequest,
    client: &Client,
) -> Result<Vec<MappedMealItem>, AppError> {
    validate_request(&request)?;
    let content = match request.provider.trim().to_ascii_lowercase().as_str() {
        "google" | "gemini" => google::generate(client, &request).await?,
        "anthropic" | "claude" => anthropic::generate(client, &request).await?,
        "openai" | "openrouter" | "custom" => openai::generate(client, &request).await?,
        _ => return Err(AppError::Validation("unsupported AI provider".into())),
    };
    let plan = prompt::parse_meal_plan(&content)?;
    let mapped = meals::map_ai_items(storage, &plan).await?;
    Ok(normalize_menu_to_tdee(mapped, request.target_tdee))
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
    endpoint(&request.base_url, "")?;
    Ok(())
}

pub fn endpoint(base_url: &str, suffix: &str) -> Result<Url, AppError> {
    let value = base_url.trim();
    let url =
        Url::parse(value).map_err(|_| AppError::Validation("AI base URL is invalid".into()))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || is_private_host(&url)
    {
        return Err(AppError::Validation("AI base URL is invalid".into()));
    }
    let suffix = suffix.trim_matches('/');
    if suffix.is_empty() {
        return Ok(url);
    }
    let existing = url.path().trim_end_matches('/');
    if existing.ends_with(&format!("/{suffix}")) || existing == suffix {
        return Ok(url);
    }
    let mut joined = url;
    let base_path = joined.path().trim_end_matches('/');
    joined.set_path(&format!("{base_path}/{suffix}"));
    Ok(joined)
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

pub(crate) fn request_error(status: reqwest::StatusCode) -> AppError {
    AppError::Ai(format!("AI provider returned HTTP {}", status.as_u16()))
}

pub(crate) fn parse_response(body: &str, field: &str) -> Result<String, AppError> {
    let value: Value = serde_json::from_str(body)
        .map_err(|_| AppError::Ai("AI provider returned malformed JSON".into()))?;
    value
        .pointer(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| AppError::Ai("AI provider response is missing content".into()))
}

pub(crate) async fn send_json<T: Serialize>(
    _client: &Client,
    request: reqwest::RequestBuilder,
    payload: &T,
) -> Result<String, AppError> {
    let response = request
        .json(payload)
        .send()
        .await
        .map_err(|_| AppError::Ai("AI provider request failed".into()))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|_| AppError::Ai("AI provider response could not be read".into()))?;
    if !status.is_success() {
        return Err(request_error(status));
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

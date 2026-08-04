use crate::{
    error::AppError,
    meals,
    models::{AiRequest, MappedMealItem},
    storage::Storage,
};
use reqwest::{Client, Url};
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

mod anthropic;
mod google;
mod openai;
pub mod prompt;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn generate_menu(
    storage: &Storage,
    request: AiRequest,
) -> Result<Vec<MappedMealItem>, AppError> {
    validate_request(&request)?;
    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|_| AppError::Ai("unable to configure AI client".into()))?;
    let content = match request.provider.trim().to_ascii_lowercase().as_str() {
        "google" | "gemini" => google::generate(&client, &request).await?,
        "anthropic" | "claude" => anthropic::generate(&client, &request).await?,
        "openai" | "openrouter" | "custom" => openai::generate(&client, &request).await?,
        _ => return Err(AppError::Validation("unsupported AI provider".into())),
    };
    let plan = prompt::parse_meal_plan(&content)?;
    meals::map_ai_items(storage, &plan).await
}

fn validate_request(request: &AiRequest) -> Result<(), AppError> {
    if request.model.trim().is_empty() || request.available_meal_types.is_empty() {
        return Err(AppError::Validation(
            "AI model and meal types are required".into(),
        ));
    }
    endpoint(&request.base_url, "")?;
    Ok(())
}

pub(crate) fn endpoint(base_url: &str, suffix: &str) -> Result<Url, AppError> {
    let value = base_url.trim();
    let url =
        Url::parse(value).map_err(|_| AppError::Validation("AI base URL is invalid".into()))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || url.username() != ""
        || url.password().is_some()
    {
        return Err(AppError::Validation("AI base URL is invalid".into()));
    }
    let normalized = value.trim_end_matches('/');
    let url = if normalized.ends_with(suffix) {
        normalized.to_string()
    } else {
        format!("{normalized}/{suffix}")
    };
    Url::parse(&url).map_err(|_| AppError::Validation("AI endpoint is invalid".into()))
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

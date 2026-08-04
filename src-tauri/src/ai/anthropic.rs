use super::{endpoint, parse_response, prompt, send_json};
use crate::{error::AppError, models::AiRequest};
use reqwest::Client;
use serde::Serialize;

pub(crate) async fn generate(client: &Client, request: &AiRequest) -> Result<String, AppError> {
    let url = endpoint(&request.base_url, "messages")?;
    let payload = AnthropicPayload {
        model: &request.model,
        max_tokens: 4096,
        temperature: 0.2,
        system: "You are a nutrition meal-planning assistant. Always return valid JSON only.",
        messages: vec![AnthropicMessage {
            role: "user",
            content: prompt::build(request),
        }],
    };
    let builder = client
        .post(url)
        .header("x-api-key", &request.api_key)
        .header("anthropic-version", "2023-06-01");
    let body = send_json(client, builder, &payload).await?;
    parse_response(&body, "/content/0/text")
}

#[derive(Debug, Serialize)]
struct AnthropicPayload<'a> {
    model: &'a str,
    max_tokens: u32,
    temperature: f64,
    system: &'static str,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: &'static str,
    content: String,
}

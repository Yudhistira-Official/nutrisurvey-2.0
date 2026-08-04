use super::{endpoint, parse_response, prompt, send_json};
use crate::{error::AppError, models::AiRequest};
use reqwest::{Client, Url};
use serde::Serialize;

pub(crate) async fn generate(client: &Client, request: &AiRequest) -> Result<String, AppError> {
    let url = google_endpoint(&request.base_url, &request.model)?;
    let payload = GooglePayload {
        contents: vec![GoogleContent {
            role: "user",
            parts: vec![GooglePart {
                text: prompt::build(request),
            }],
        }],
        generation_config: GenerationConfig { temperature: 0.2 },
    };
    let body = send_json(
        client,
        client.post(url).header("x-goog-api-key", &request.api_key),
        &payload,
    )
    .await?;
    parse_response(&body, "/candidates/0/content/parts/0/text")
}

pub fn google_endpoint(base_url: &str, model: &str) -> Result<Url, AppError> {
    let base = endpoint(base_url, "")?;
    let encoded_model = percent_encode(model);
    endpoint(
        base.as_str(),
        &format!("models/{encoded_model}:generateContent"),
    )
}

fn percent_encode(value: &str) -> String {
    value.bytes().fold(String::new(), |mut encoded, byte| {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
        encoded
    })
}

#[derive(Debug, Serialize)]
struct GooglePayload {
    contents: Vec<GoogleContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
struct GoogleContent {
    role: &'static str,
    parts: Vec<GooglePart>,
}

#[derive(Debug, Serialize)]
struct GooglePart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f64,
}

use super::{endpoint, endpoint_for_client, parse_response, prompt, send_json};
use crate::{error::AppError, models::AiRequest};
use reqwest::{Client, Url};
use serde::Serialize;

pub(crate) async fn generate(client: &Client, request: &AiRequest) -> Result<String, AppError> {
    let url = google_endpoint_for_client(&request.base_url, &request.model)?;
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
    join_google_endpoint(base, model)
}

pub fn google_endpoint_for_client(base_url: &str, model: &str) -> Result<Url, AppError> {
    let base = endpoint_for_client(base_url, "")?;
    join_google_endpoint(base, model)
}

fn join_google_endpoint(base: Url, model: &str) -> Result<Url, AppError> {
    let encoded_model = percent_encode(model);
    join_endpoint_for_client(base, &format!("models/{encoded_model}:generateContent"))
}

fn join_endpoint_for_client(mut base: Url, suffix: &str) -> Result<Url, AppError> {
    let base_path = base.path().trim_end_matches('/');
    base.set_path(&format!("{base_path}/{suffix}"));
    Ok(base)
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

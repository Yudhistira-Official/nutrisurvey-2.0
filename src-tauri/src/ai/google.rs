use super::{endpoint, parse_response, prompt, send_json};
use crate::{error::AppError, models::AiRequest};
use reqwest::Client;
use serde::Serialize;

pub(crate) async fn generate(client: &Client, request: &AiRequest) -> Result<String, AppError> {
    let suffix = format!("models/{}:generateContent", request.model);
    let url = endpoint(&request.base_url, &suffix)?;
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

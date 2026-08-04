use super::{endpoint, parse_response, payload, send_json};
use crate::{error::AppError, models::AiRequest};
use reqwest::Client;
use serde::Serialize;

pub(crate) async fn generate(client: &Client, request: &AiRequest) -> Result<String, AppError> {
    let url = endpoint(&request.base_url, "chat/completions")?;
    let mut builder = client.post(url);
    if !request.api_key.trim().is_empty() {
        builder = builder.bearer_auth(&request.api_key);
    }
    let body = send_json(client, builder, &payload(request)).await?;
    parse_response(&body, "/choices/0/message/content")
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct _Marker;

use super::{endpoint_for_client as endpoint, parse_response, payload, send_json};
use crate::{error::AppError, models::AiRequest};
use futures_util::StreamExt;
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

pub(crate) async fn generate_stream<F>(
    client: &Client,
    request: &AiRequest,
    mut on_token: F,
) -> Result<String, AppError>
where
    F: FnMut(&str),
{
    let url = endpoint(&request.base_url, "chat/completions")?;
    let mut stream_payload = serde_json::to_value(payload(request))
        .map_err(|_| AppError::Ai("failed to serialize payload".into()))?;
    stream_payload["stream"] = serde_json::Value::Bool(true);
    let mut builder = client.post(url).json(&stream_payload);
    if !request.api_key.trim().is_empty() {
        builder = builder.bearer_auth(&request.api_key);
    }
    let response = builder
        .send()
        .await
        .map_err(|_| AppError::Ai("AI provider request failed".into()))?;
    let status = response.status();
    if !status.is_success() {
        return Err(super::request_error(status));
    }
    let mut stream = response.bytes_stream();
    let mut full = String::new();
    let mut buf = String::new();
    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|_| AppError::Ai("stream read error".into()))?;
        buf.push_str(&String::from_utf8_lossy(&bytes));
        let mut start = 0;
        while let Some(nl) = buf[start..].find('\n') {
            let line = buf[start..start + nl].trim().to_owned();
            start += nl + 1;
            if let Some(data) = line.strip_prefix("data: ") {
                if data == "[DONE]" {
                    continue;
                }
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(token) = val
                        .pointer("/choices/0/delta/content")
                        .and_then(|v| v.as_str())
                    {
                        full.push_str(token);
                        on_token(token);
                    }
                }
            }
        }
        buf = buf[start..].to_owned();
    }
    Ok(full)
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct _Marker;

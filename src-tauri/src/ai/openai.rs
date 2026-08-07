use super::{endpoint_for_client as endpoint, is_cancelled, payload, send_json};
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
    let body = send_json(client, builder, &payload(request), &request.api_key).await?;
    super::parse_any_response(
        &body,
        &[
            "/choices/0/message/content",
            "/output/0/content/0/text",
            "/output_text",
            "/content/0/text",
        ],
    )
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
    let mut builder = client
        .post(url)
        .header(reqwest::header::ACCEPT_ENCODING, "identity")
        .json(&stream_payload);
    if !request.api_key.trim().is_empty() {
        builder = builder.bearer_auth(&request.api_key);
    }
    let response = builder
        .send()
        .await
        .map_err(|_| AppError::Ai("AI provider request failed".into()))?;
    let status = response.status();
    if !status.is_success() {
        let body =
            String::from_utf8_lossy(&response.bytes().await.unwrap_or_default()).into_owned();
        return Err(super::request_error(status, &body, &request.api_key));
    }
    let mut stream = response.bytes_stream();
    let mut full = String::new();
    let mut buf = String::new();
    while let Some(chunk) = stream.next().await {
        if is_cancelled(&request.request_id) {
            return Err(AppError::Ai("AI request cancelled".into()));
        }
        let bytes = chunk.map_err(|_| AppError::Ai("stream read error".into()))?;
        buf.push_str(&String::from_utf8_lossy(&bytes));
        process_sse_lines(&mut buf, &mut full, &mut on_token);
    }
    if !buf.trim().is_empty() {
        process_sse_line(buf.trim(), &mut full, &mut on_token);
    }
    if full.trim().is_empty() {
        return Err(AppError::Ai(
            "AI provider returned no streamed content".into(),
        ));
    }
    Ok(full)
}

fn process_sse_lines<F>(buf: &mut String, full: &mut String, on_token: &mut F)
where
    F: FnMut(&str),
{
    while let Some(nl) = buf.find('\n') {
        let line = buf[..nl].trim().to_owned();
        *buf = buf[nl + 1..].to_owned();
        process_sse_line(&line, full, on_token);
    }
}

fn process_sse_line<F>(line: &str, full: &mut String, on_token: &mut F)
where
    F: FnMut(&str),
{
    let Some(data) = line.strip_prefix("data:").map(str::trim) else {
        return;
    };
    if data == "[DONE]" {
        return;
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

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct _Marker;

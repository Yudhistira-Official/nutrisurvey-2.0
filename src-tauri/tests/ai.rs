use nutrisurvey_lib::{ai, models::AiRequest, storage::Storage};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

static IDS: AtomicUsize = AtomicUsize::new(0);

async fn storage() -> Storage {
    let id = IDS.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("nutrisurvey-ai-{id}.sqlite"));
    let _ = tokio::fs::remove_file(&path).await;
    Storage::open_path(&path).await.unwrap()
}

async fn mock_server(status: u16, body: &'static str) -> (String, tokio::task::JoinHandle<String>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        let mut buffer = [0; 4096];
        loop {
            let count = socket.read(&mut buffer).await.unwrap();
            if count == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..count]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        let response = format!(
            "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).await.unwrap();
        String::from_utf8(request).unwrap()
    });
    (url, task)
}

fn request(base_url: String, provider: &str) -> AiRequest {
    AiRequest {
        target_tdee: 2000,
        target_carbs: 250,
        target_protein: 100,
        target_fat: 60,
        available_meal_types: vec!["Sarapan".into()],
        provider: provider.into(),
        model: "test-model".into(),
        api_key: "super-secret-key".into(),
        base_url,
    }
}

const PLAN: &str = r#"{"meal_plan":[{"meal_type":"Sarapan","food_keyword":"Nasi","suggested_grams":100,"reasoning":"seimbang"}]}"#;

#[tokio::test]
async fn openai_compatible_provider_posts_schema_prompt_and_parses_plan() {
    let storage = storage().await;
    let (url, task) = mock_server(200, r#"{"choices":[{"message":{"content":"{\"meal_plan\":[{\"meal_type\":\"Sarapan\",\"food_keyword\":\"Nasi\",\"suggested_grams\":100,\"reasoning\":\"seimbang\"}]}"}}]}"#).await;
    let result = ai::generate_menu(&storage, request(url, "openai")).await;
    assert!(result.is_ok());
    let raw_request = task.await.unwrap();
    assert!(raw_request.contains("/chat/completions"));
    assert!(raw_request.contains("Bearer super-secret-key"));
    assert!(raw_request.contains("meal_plan"));
}

#[tokio::test]
async fn google_provider_uses_supported_key_header_and_fenced_json() {
    let storage = storage().await;
    let body =
        r#"{"candidates":[{"content":{"parts":[{"text":"```json\n{\"meal_plan\":[]}\n```"}]}}]}"#;
    let (url, task) = mock_server(200, body).await;
    let result = ai::generate_menu(&storage, request(url, "google")).await;
    assert!(result.unwrap().is_empty());
    let raw_request = task.await.unwrap();
    assert!(raw_request.contains("/models/test-model:generateContent"));
    assert!(raw_request.contains("x-goog-api-key: super-secret-key"));
    assert!(!raw_request.contains("?key=super-secret-key"));
}

#[tokio::test]
async fn anthropic_provider_parses_response() {
    let storage = storage().await;
    let body = format!(r#"{{"content":[{{"text":{PLAN:?}}}]}}"#);
    let body: &'static str = Box::leak(body.into_boxed_str());
    let (url, task) = mock_server(200, body).await;
    let result = ai::generate_menu(&storage, request(url, "anthropic")).await;
    assert!(result.is_ok());
    assert!(task.await.unwrap().contains("/messages"));
}

#[tokio::test]
async fn malformed_json_missing_fields_and_http_errors_are_redacted() {
    let storage = storage().await;
    for body in ["not-json", r#"{"choices":[]}"#] {
        let (url, task) = mock_server(200, Box::leak(body.to_string().into_boxed_str())).await;
        let error = ai::generate_menu(&storage, request(url, "openrouter"))
            .await
            .unwrap_err()
            .to_string();
        assert!(!error.contains("super-secret-key"));
        task.await.unwrap();
    }
    let (url, task) = mock_server(500, r#"{"error":"super-secret-key"}"#).await;
    let error = ai::generate_menu(&storage, request(url, "custom"))
        .await
        .unwrap_err()
        .to_string();
    assert!(!error.contains("super-secret-key"));
    assert!(error.contains("HTTP 500"));
    task.await.unwrap();
}

#[tokio::test]
async fn provider_url_validation_rejects_unsafe_urls_without_key_leak() {
    let storage = storage().await;
    let mut request = request("file:///tmp/secret".into(), "openai");
    let error = ai::generate_menu(&storage, request.clone())
        .await
        .unwrap_err()
        .to_string();
    assert!(!error.contains(&request.api_key));
    request.base_url = "https://user:password@example.test".into();
    let error = ai::generate_menu(&storage, request)
        .await
        .unwrap_err()
        .to_string();
    assert!(!error.contains("password"));
}

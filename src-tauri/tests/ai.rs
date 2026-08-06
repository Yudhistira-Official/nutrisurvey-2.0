use nutrisurvey_lib::{
    ai, foods,
    models::{AiRequest, MappedMealItem},
    storage::Storage,
};
use reqwest::Client;
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

async fn mock_server(
    status: u16,
    body: &'static str,
) -> (String, tokio::task::JoinHandle<String>, Client) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://example.com:{port}");
    let client = Client::builder()
        .resolve("example.com", listener.local_addr().unwrap())
        .build()
        .unwrap();
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
    (url, task, client)
}

fn request(base_url: String, provider: &str) -> AiRequest {
    AiRequest {
        target_tdee: 2000,
        target_carbs: 250,
        target_protein: 100,
        target_fat: 60,
        prompt: "menu rendah gula".into(),
        active_menu: "[]".into(),
        revision: false,
        verify_menu: false,
        candidate_catalog: String::new(),
        request_id: String::new(),
        available_meal_types: vec!["Sarapan".into()],
        provider: provider.into(),
        model: "test-model".into(),
        api_key: "super-secret-key".into(),
        base_url,
    }
}

const PLAN: &str = r#"{"meal_plan":[{"meal_type":"Sarapan","food_keyword":"Nasi","suggested_grams":100,"reasoning":"seimbang"}]}"#;

#[test]
fn zero_macro_foods_are_excluded_except_water() {
    assert!(foods::is_nutritionally_empty(
        "Snack",
        "Makanan",
        &std::collections::HashMap::from([
            ("energi".into(), 0.0),
            ("protein".into(), 0.0),
            ("lemak total".into(), 0.0),
            ("karbohidrat total".into(), 0.0),
        ])
    ));
    assert!(!foods::is_nutritionally_empty(
        "Snack",
        "Makanan",
        &std::collections::HashMap::new()
    ));
    assert!(foods::is_allowed_zero_food(
        "Air putih / air mineral / air minum"
    ));
    assert!(!foods::is_allowed_zero_food("Teh tawar"));
}

#[test]
fn prompt_includes_database_candidate_catalog() {
    let mut request = request("https://example.test".into(), "openai");
    request.candidate_catalog = "Nasi | serving 100 g | energi 130".into();
    let prompt = ai::prompt::build(&request);
    assert!(prompt.contains("KATALOG DATABASE SQLITE"));
    assert!(prompt.contains("Nasi | serving 100 g"));
}

#[test]
fn macro_verification_reports_target_gaps() {
    let items = vec![MappedMealItem {
        meal_type: "Sarapan".into(),
        requested_keyword: "Nasi".into(),
        matched_food_id: 1,
        matched_food_name: "Nasi".into(),
        suggested_grams: 100,
        reference_grams: 100.0,
        calories: 400.0,
        protein: 10.0,
        fat: 5.0,
        carbohydrate: 70.0,
        nutrients: std::collections::HashMap::new(),
        reasoning: "".into(),
    }];
    let report = ai::verify_menu_targets(&items, 2000, 250, 100, 60);
    assert!(!report.is_valid);
    assert!(report.feedback.contains("Protein"));
    assert!(report.feedback.contains("Karbohidrat"));
}

#[tokio::test]
async fn openai_compatible_provider_posts_schema_prompt_and_parses_plan() {
    let storage = storage().await;
    let (url, task, client) = mock_server(200, r#"{"choices":[{"message":{"content":"{\"meal_plan\":[{\"meal_type\":\"Sarapan\",\"food_keyword\":\"Nasi\",\"suggested_grams\":100,\"reasoning\":\"seimbang\"}]}"}}]}"#).await;
    let result = ai::generate_menu_with_client(&storage, request(url, "openai"), &client).await;
    assert!(result.is_ok());
    let raw_request = task.await.unwrap();
    assert!(raw_request.contains("/chat/completions"));
    assert!(raw_request.contains("Bearer super-secret-key"));
    assert!(raw_request.contains("meal_plan"));
}

#[tokio::test]
async fn google_provider_uses_supported_key_header_and_fenced_json() {
    let storage = storage().await;
    let body = r#"{"candidates":[{"content":{"parts":[{"text":"```json\n{\"meal_plan\":[{\"meal_type\":\"Sarapan\",\"food_keyword\":\"Nasi\",\"suggested_grams\":100,\"reasoning\":\"seimbang\"}]}\n```"}]}}]}"#;
    let (url, task, client) = mock_server(200, body).await;
    let result = ai::generate_menu_with_client(&storage, request(url, "google"), &client).await;
    assert!(result.is_ok());
    let raw_request = task.await.unwrap();
    assert!(raw_request.contains("/models/test-model:generateContent"));
    assert!(raw_request.contains("x-goog-api-key: super-secret-key"));
    assert!(!raw_request.contains("?key=super-secret-key"));
}

#[tokio::test]
async fn verified_generation_retries_at_most_three_times() {
    let report = ai::verify_menu_targets(&[], 2000, 250, 100, 60);
    assert!(!report.is_valid);
    assert!(report.error_score > 0.0);
}

#[tokio::test]
async fn generate_menu_normalizes_tdee_and_drops_items_below_25_grams() {
    let storage = storage().await;
    sqlx::query("INSERT INTO foods (id,name,normalized_name,serving_size,serving_unit,servings_per_container) VALUES (1,'Nasi','nasi',100,'g',1),(2,'Telur','telur',100,'g',1)")
        .execute(storage.pool())
        .await
        .unwrap();
    for (id, name, amount) in [(1, "Energi", 100.0), (2, "Protein", 10.0)] {
        sqlx::query("INSERT INTO nutrients (id,name,normalized_name,unit) VALUES (?,?,?,?)")
            .bind(id)
            .bind(name)
            .bind(name.to_lowercase())
            .bind("g")
            .execute(storage.pool())
            .await
            .unwrap();
        for food_id in 1..=2 {
            sqlx::query("INSERT INTO food_nutrients (food_id,nutrient_id,amount) VALUES (?,?,?)")
                .bind(food_id)
                .bind(id)
                .bind(amount)
                .execute(storage.pool())
                .await
                .unwrap();
        }
    }
    let response = r#"{"choices":[{"message":{"content":"{\"meal_plan\":[{\"meal_type\":\"Sarapan\",\"food_keyword\":\"Nasi\",\"suggested_grams\":100,\"reasoning\":\"utama\"},{\"meal_type\":\"Sarapan\",\"food_keyword\":\"Telur\",\"suggested_grams\":10,\"reasoning\":\"kecil\"}]}"}}]}"#;
    let (url, task, client) = mock_server(200, response).await;
    let mut request = request(url, "openai");
    request.target_tdee = 210;
    let mapped = ai::generate_menu_with_client(&storage, request, &client)
        .await
        .unwrap();
    assert_eq!(mapped.len(), 1);
    assert_eq!(mapped[0].suggested_grams, 191);
    assert_eq!(mapped[0].calories, 191.0);
    task.await.unwrap();
}

#[tokio::test]
async fn anthropic_provider_parses_response() {
    let storage = storage().await;
    let body = format!(r#"{{"content":[{{"text":{PLAN:?}}}]}}"#);
    let body: &'static str = Box::leak(body.into_boxed_str());
    let (url, task, client) = mock_server(200, body).await;
    let result = ai::generate_menu_with_client(&storage, request(url, "anthropic"), &client).await;
    assert!(result.is_ok());
    assert!(task.await.unwrap().contains("/messages"));
}

#[tokio::test]
async fn malformed_json_missing_fields_and_http_errors_are_redacted() {
    let storage = storage().await;
    for body in ["not-json", r#"{"choices":[]}"#] {
        let (url, task, client) =
            mock_server(200, Box::leak(body.to_string().into_boxed_str())).await;
        let error = ai::generate_menu_with_client(&storage, request(url, "openrouter"), &client)
            .await
            .unwrap_err()
            .to_string();
        assert!(!error.contains("super-secret-key"));
        task.await.unwrap();
    }
    let (url, task, client) = mock_server(500, r#"{"error":"super-secret-key"}"#).await;
    let error = ai::generate_menu_with_client(&storage, request(url, "custom"), &client)
        .await
        .unwrap_err()
        .to_string();
    assert!(!error.contains("super-secret-key"));
    assert!(error.contains("HTTP 500"));
    task.await.unwrap();
}

#[test]
fn verification_retry_does_not_mix_attempt_output() {
    let source = std::fs::read_to_string("src/ai/mod.rs").unwrap();
    assert!(source.contains("if attempt == 0"));
    assert!(source.contains("parse_meal_plan"));
}

#[test]
fn response_parser_accepts_common_json_noise() {
    let content = "Here is plan:\n```json\n{\"meal_plan\":[{\"meal_type\":\"Sarapan\",\"food_keyword\":\"Nasi\",\"suggested_grams\":100,\"reasoning\":\"ok\",}] }\n```\n";
    assert_eq!(ai::prompt::parse_meal_plan(content).unwrap().len(), 1);
}

#[test]
fn response_parser_accepts_array_and_output_text_formats() {
    let array = r#"{"choices":[{"message":{"content":[{"type":"text","text":"hello"}]}}]}"#;
    assert_eq!(
        ai::parse_any_response(array, &["/choices/0/message/content"]).unwrap(),
        "hello"
    );
    let output = r#"{"output_text":"hello"}"#;
    assert_eq!(
        ai::parse_any_response(output, &["/output_text"]).unwrap(),
        "hello"
    );
}

#[test]
fn provider_errors_keep_safe_server_detail() {
    let error = ai::request_error(
        reqwest::StatusCode::TOO_MANY_REQUESTS,
        r#"{"error":{"message":"quota exceeded"}}"#,
        "super-secret-key",
    )
    .to_string();
    assert!(error.contains("HTTP 429"));
    assert!(error.contains("quota exceeded"));
    assert!(!error.contains("super-secret-key"));
}

#[test]
fn ai_request_debug_redacts_api_key() {
    let debug = format!("{:?}", request("https://example.test".into(), "openai"));
    assert!(!debug.contains("super-secret-key"));
}

#[test]
fn production_resolution_returns_hostname_url_and_pinned_safe_socket() {
    let (url, socket) = ai::resolve_and_pin("https://router.test/api", || {
        Ok::<Vec<std::net::IpAddr>, std::io::Error>(vec!["93.184.216.34".parse().unwrap()])
    })
    .unwrap();
    assert_eq!(url.host_str(), Some("router.test"));
    assert_eq!(
        socket.ip(),
        "93.184.216.34".parse::<std::net::IpAddr>().unwrap()
    );
    assert_eq!(socket.port(), 443);
}

#[test]
fn production_resolution_rejects_private_socket_before_client_creation() {
    let result = ai::resolve_and_pin("https://router.test", || {
        Ok::<Vec<std::net::IpAddr>, std::io::Error>(vec!["127.0.0.1".parse().unwrap()])
    });
    assert!(result.is_err());
}

#[test]
fn hostname_resolution_rejects_any_private_result_before_request() {
    let safe = ai::endpoint_with_resolver("https://router.test/api", "chat/completions", || {
        Ok::<Vec<std::net::IpAddr>, std::io::Error>(vec!["93.184.216.34".parse().unwrap()])
    });
    assert!(safe.is_ok());

    let unsafe_result =
        ai::endpoint_with_resolver("https://router.test/api", "chat/completions", || {
            Ok::<Vec<std::net::IpAddr>, std::io::Error>(vec![
                "93.184.216.34".parse().unwrap(),
                "127.0.0.1".parse().unwrap(),
            ])
        });
    assert!(unsafe_result.is_err());
}

#[test]
fn hostname_resolution_rejects_unresolved_host() {
    let result = ai::endpoint_with_resolver("https://router.test", "", || {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "missing"))
    });
    assert!(result.is_err());
}

#[test]
fn endpoint_validation_rejects_unsafe_urls_and_structurally_joins_paths() {
    for base_url in [
        "https://example.test?token=secret",
        "https://example.test/#fragment",
        "https://user:password@example.test",
        "http://10.0.0.1",
        "http://192.168.1.1",
        "http://169.254.169.254",
        "http://127.0.0.1",
        "http://127.42.0.1:8080/path",
        "http://[::1]/api",
        "http://[::ffff:127.0.0.1]/api",
    ] {
        assert!(
            ai::endpoint(base_url, "messages").is_err(),
            "accepted {base_url}"
        );
    }
    let endpoint = ai::endpoint_for_client("https://example.test/api/v1/", "messages").unwrap();
    assert_eq!(endpoint.as_str(), "https://example.test/api/v1/messages");
    let existing =
        ai::endpoint_for_client("https://example.test/api/messages", "messages").unwrap();
    assert_eq!(existing.as_str(), "https://example.test/api/messages");
}

#[test]
fn google_model_path_is_url_encoded() {
    assert_eq!(
        ai::google_endpoint_for_client("https://example.test/api", "models/a b/v1")
            .unwrap()
            .as_str(),
        "https://example.test/api/models/models%2Fa%20b%2Fv1:generateContent"
    );
}

#[test]
fn parser_accepts_case_insensitive_crlf_fences_and_surrounding_prose() {
    let content = format!("intro\r\n```JSON\r\n{}\r\n```\r\noutro", PLAN);
    assert_eq!(ai::prompt::parse_meal_plan(&content).unwrap().len(), 1);
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

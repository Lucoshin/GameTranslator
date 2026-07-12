use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
};

use game_translator_provider_core::{
    OllamaProvider, OpenAiCompatibleProvider, ProviderError, TranslationInput, TranslationProvider,
    TranslationRequest,
};

fn mock_server(status: u16, body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 8192];
        let _ = stream.read(&mut request).unwrap();
        let reason = if status == 200 {
            "OK"
        } else {
            "Too Many Requests"
        };
        write!(
            stream,
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
    });
    format!("http://{address}")
}

fn capturing_mock_server(body: &'static str) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 16_384];
        let length = stream.read(&mut request).unwrap();
        sender
            .send(String::from_utf8_lossy(&request[..length]).into_owned())
            .unwrap();
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
    });
    (format!("http://{address}"), receiver)
}

fn request() -> TranslationRequest {
    TranslationRequest {
        model: "test-model".into(),
        source_language: "ja-JP".into(),
        target_language: "zh-CN".into(),
        segments: vec![TranslationInput {
            id: "segment-1".into(),
            text: "Moonstone".into(),
        }],
    }
}

#[test]
fn openai_compatible_provider_reads_structured_translations() {
    let content = r#"{\"translations\":[{\"id\":\"segment-1\",\"text\":\"月光石\"}]}"#;
    let body = format!(r#"{{"choices":[{{"message":{{"content":"{content}"}}}}]}}"#);
    let provider =
        OpenAiCompatibleProvider::new(mock_server(200, Box::leak(body.into_boxed_str())), "key");

    let response = provider.translate(&request()).unwrap();

    assert_eq!(response.translations[0].id, "segment-1");
    assert_eq!(response.translations[0].text, "月光石");
}

#[test]
fn ollama_provider_reads_structured_translations() {
    let content = r#"{\"translations\":[{\"id\":\"segment-1\",\"text\":\"月光石\"}]}"#;
    let body = format!(r#"{{"message":{{"content":"{content}"}}}}"#);
    let provider = OllamaProvider::new(mock_server(200, Box::leak(body.into_boxed_str())));

    let response = provider.translate(&request()).unwrap();

    assert_eq!(response.translations[0].text, "月光石");
}

#[test]
fn maps_http_429_to_rate_limited() {
    let provider =
        OpenAiCompatibleProvider::new(mock_server(429, r#"{"error":"slow down"}"#), "key");

    assert_eq!(
        provider.translate(&request()).unwrap_err(),
        ProviderError::RateLimited
    );
}

#[test]
fn deepseek_requests_use_a_stable_system_prefix_and_user_isolation() {
    let content = r#"{\"translations\":[{\"id\":\"segment-1\",\"text\":\"月光石\"}]}"#;
    let response = Box::leak(
        format!(r#"{{"choices":[{{"message":{{"content":"{content}"}}}}]}}"#).into_boxed_str(),
    );
    let (url, captured) = capturing_mock_server(response);
    let provider =
        OpenAiCompatibleProvider::new(url, "key").with_user_id("game-translator-desktop");

    provider.translate(&request()).unwrap();

    let raw = captured.recv().unwrap();
    let payload = raw.split("\r\n\r\n").nth(1).unwrap();
    let json: serde_json::Value = serde_json::from_str(payload).unwrap();
    assert_eq!(json["messages"][0]["role"], "system");
    assert!(json["messages"][0]["content"]
        .as_str()
        .unwrap()
        .contains("game localization translator"));
    assert_eq!(json["messages"][1]["role"], "user");
    assert_eq!(json["user_id"], "game-translator-desktop");
}

#[test]
fn accepts_common_deepseek_translation_text_aliases() {
    for field in ["translated", "translated_text", "translatedText"] {
        let content =
            format!(r#"{{\"translations\":[{{\"id\":\"segment-1\",\"{field}\":\"月光石\"}}]}}"#);
        let body = format!(r#"{{"choices":[{{"message":{{"content":"{content}"}}}}]}}"#);
        let provider = OpenAiCompatibleProvider::new(
            mock_server(200, Box::leak(body.into_boxed_str())),
            "key",
        );

        let response = provider.translate(&request()).unwrap();

        assert_eq!(response.translations[0].text, "月光石");
    }
}

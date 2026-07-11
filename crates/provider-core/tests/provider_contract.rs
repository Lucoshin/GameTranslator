use std::{
    io::{Read, Write},
    net::TcpListener,
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

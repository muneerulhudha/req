use req::app::{App, EditorField};

#[test]
fn editor_field_cycle_roundtrip() {
    let mut field = EditorField::Name;
    for _ in 0..7 {
        field = field.next();
    }
    assert!(matches!(field, EditorField::Name));
}

#[test]
fn method_cycle_advances() {
    let mut app = App::new();
    app.request.method = "GET".to_string();
    app.cycle_method();
    assert_eq!(app.request.method, "POST");
}

#[test]
fn curl_preview_populates_response() {
    let mut app = App::new();
    app.request.method = "POST".to_string();
    app.request.url = "https://example.test/items".to_string();
    app.request.body = "{\"x\":1}".to_string();
    app.curl_preview();

    let resp = app.response.expect("response should be present");
    assert_eq!(resp.reason, "cURL Preview");
    assert!(resp.body.contains("curl -X POST"));
}

use req::util::{apply_query, interpolate, parse_map, pretty_body};

#[test]
fn parse_map_supports_colon_and_equals() {
    let raw = "Authorization: Bearer abc\npage=10\n# ignore";
    let map = parse_map(raw);
    assert_eq!(map.get("Authorization").unwrap(), "Bearer abc");
    assert_eq!(map.get("page").unwrap(), "10");
}

#[test]
fn interpolate_replaces_known_vars() {
    let env = parse_map("base=https://api.test\ntoken=xyz");
    assert_eq!(
        interpolate("{{base}}/users", &env),
        "https://api.test/users"
    );
}

#[test]
fn apply_query_encodes_parameters() {
    let params = parse_map("q=hello world\npage=1");
    let url = apply_query("https://x.test/search", &params);
    assert!(url.contains("q=hello%20world"));
    assert!(url.contains("page=1"));
}

#[test]
fn pretty_json_formats_body() {
    let out = pretty_body("{\"ok\":true}", "application/json");
    assert!(out.contains("\"ok\": true"));
}

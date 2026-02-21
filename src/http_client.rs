use std::time::Duration;

use anyhow::Result;
use reqwest::blocking::Client;

use crate::{
    model::{ApiRequest, ApiResponse},
    util::{apply_query, interpolate, parse_map, pretty_body},
};

pub fn execute_request(request: &ApiRequest, env_raw: &str) -> Result<ApiResponse> {
    let env = parse_map(env_raw);
    let method = request.method.parse::<reqwest::Method>()?;
    let url = apply_query(
        &interpolate(&request.url, &env),
        &parse_map(&request.params_raw),
    );
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    let mut builder = client.request(method, url);
    for (k, v) in parse_map(&request.headers_raw) {
        builder = builder.header(k, interpolate(&v, &env));
    }
    if !request.body.trim().is_empty() {
        builder = builder.body(interpolate(&request.body, &env));
    }

    let start = std::time::Instant::now();
    let response = builder.send()?;
    let elapsed_ms = start.elapsed().as_millis();
    let status = response.status();
    let reason = status.canonical_reason().unwrap_or("Unknown").to_string();
    let headers = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("<binary>").to_string()))
        .collect::<Vec<_>>();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("text/plain")
        .to_string();
    let body = response.text()?;

    Ok(ApiResponse {
        status_code: status.as_u16(),
        reason,
        elapsed_ms,
        content_type: content_type.clone(),
        headers,
        body: pretty_body(&body, &content_type),
    })
}

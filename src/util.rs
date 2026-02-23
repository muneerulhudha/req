use std::collections::BTreeMap;

pub fn parse_map(raw: &str) -> BTreeMap<String, String> {
    raw.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (k, v) = line.split_once('=').or_else(|| line.split_once(':'))?;
            Some((k.trim().to_string(), v.trim().to_string()))
        })
        .collect()
}

pub fn interpolate(value: &str, env: &BTreeMap<String, String>) -> String {
    let mut out = value.to_string();
    for (k, v) in env {
        out = out.replace(&format!("{{{{{k}}}}}"), v);
    }
    out
}

pub fn apply_query(url: &str, params: &BTreeMap<String, String>) -> String {
    if params.is_empty() {
        return url.to_string();
    }
    let encoded = params
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    let sep = if url.contains('?') { '&' } else { '?' };
    format!("{url}{sep}{encoded}")
}

pub fn pretty_body(body: &str, content_type: &str) -> String {
    if content_type.to_ascii_lowercase().contains("json") {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(body) {
            return serde_json::to_string_pretty(&parsed).unwrap_or_else(|_| body.to_string());
        }
    }
    body.to_string()
}

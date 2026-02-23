use chrono::Utc;

use crate::{
    http_client::execute_request,
    model::{ApiRequest, ApiResponse, AppState, HistoryEntry},
    storage::{load_state, save_state},
    util::{apply_query, interpolate, parse_map},
};

const METHODS: [&str; 9] = [
    "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "TRACE", "CONNECT",
];

const BODY_FORMATS: [(&str, &str, Option<&str>); 9] = [
    ("none", "None", None),
    ("json", "JSON", Some("application/json")),
    ("text", "Plain text", Some("text/plain")),
    ("xml", "XML", Some("application/xml")),
    ("html", "HTML", Some("text/html")),
    (
        "form_urlencoded",
        "Form URL Encoded",
        Some("application/x-www-form-urlencoded"),
    ),
    (
        "multipart",
        "Multipart form-data",
        Some("multipart/form-data"),
    ),
    ("octet_stream", "Binary", Some("application/octet-stream")),
    ("custom", "Custom", None),
];

const COMMON_HEADERS: [&str; 11] = [
    "Accept",
    "Accept-Encoding",
    "Accept-Language",
    "Authorization",
    "Cache-Control",
    "Connection",
    "Content-Type",
    "Cookie",
    "If-None-Match",
    "User-Agent",
    "X-Request-ID",
];

#[derive(Debug, Clone, Copy)]
pub enum Focus {
    Collections,
    History,
    Editor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorField {
    Name,
    Method,
    Url,
    Params,
    HeaderKey,
    HeaderValue,
    BodyFormat,
    CustomContentType,
    Body,
    Env,
}

impl EditorField {
    pub fn next(self) -> Self {
        use EditorField::*;
        match self {
            Name => Method,
            Method => Url,
            Url => Params,
            Params => HeaderKey,
            HeaderKey => HeaderValue,
            HeaderValue => BodyFormat,
            BodyFormat => CustomContentType,
            CustomContentType => Body,
            Body => Env,
            Env => Name,
        }
    }

    pub fn prev(self) -> Self {
        use EditorField::*;
        match self {
            Name => Env,
            Method => Name,
            Url => Method,
            Params => Url,
            HeaderKey => Params,
            HeaderValue => HeaderKey,
            BodyFormat => HeaderValue,
            CustomContentType => BodyFormat,
            Body => CustomContentType,
            Env => Body,
        }
    }
}

pub struct App {
    pub state: AppState,
    pub request: ApiRequest,
    pub response: Option<ApiResponse>,
    pub response_mode_headers: bool,
    pub status: String,
    pub focus: Focus,
    pub editor_field: EditorField,
    pub collections_idx: usize,
    pub history_idx: usize,
    pub input_mode: bool,
    pub input_buffer: String,
    pub header_key_input: String,
    pub header_value_input: String,
    pub header_suggestion_idx: usize,
}

impl App {
    pub fn new() -> Self {
        let state = load_state().unwrap_or_default();
        Self {
            request: ApiRequest::default(),
            state,
            response: None,
            response_mode_headers: false,
            status: "Ready — req (terminal API client)".into(),
            focus: Focus::Editor,
            editor_field: EditorField::Url,
            collections_idx: 0,
            history_idx: 0,
            input_mode: false,
            input_buffer: String::new(),
            header_key_input: String::new(),
            header_value_input: String::new(),
            header_suggestion_idx: 0,
        }
    }

    pub fn cycle_method(&mut self) {
        let pos = METHODS
            .iter()
            .position(|m| *m == self.request.method)
            .unwrap_or(0);
        self.request.method = METHODS[(pos + 1) % METHODS.len()].to_string();
    }

    pub fn cycle_body_format(&mut self) {
        let pos = BODY_FORMATS
            .iter()
            .position(|(id, _, _)| *id == self.request.body_format)
            .unwrap_or(0);
        self.request.body_format = BODY_FORMATS[(pos + 1) % BODY_FORMATS.len()].0.to_string();
        self.sync_content_type_header();
    }

    pub fn cycle_common_header(&mut self) {
        self.header_suggestion_idx = (self.header_suggestion_idx + 1) % COMMON_HEADERS.len();
        self.header_key_input = COMMON_HEADERS[self.header_suggestion_idx].to_string();
    }

    pub fn upsert_header(&mut self) {
        let key = self.header_key_input.trim();
        if key.is_empty() {
            self.status = "Header key is required".into();
            return;
        }

        let value = self.header_value_input.trim();
        let mut headers: Vec<(String, String)> =
            parse_map(&self.request.headers_raw).into_iter().collect();
        let mut replaced = false;

        for (existing_key, existing_value) in &mut headers {
            if existing_key.eq_ignore_ascii_case(key) {
                *existing_key = key.to_string();
                *existing_value = value.to_string();
                replaced = true;
                break;
            }
        }

        if !replaced {
            headers.push((key.to_string(), value.to_string()));
        }

        self.request.headers_raw = headers
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join("\n");
        self.status = if replaced {
            format!("Updated header: {key}")
        } else {
            format!("Added header: {key}")
        };
    }

    pub fn remove_header(&mut self) {
        let key = self.header_key_input.trim();
        if key.is_empty() {
            self.status = "Header key is required".into();
            return;
        }

        let before = parse_map(&self.request.headers_raw);
        let after = before
            .iter()
            .filter(|(k, _)| !k.eq_ignore_ascii_case(key))
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join("\n");

        if after.lines().count() == before.len() {
            self.status = format!("Header not found: {key}");
            return;
        }

        self.request.headers_raw = after;
        self.status = format!("Removed header: {key}");
    }

    pub fn method_options(&self) -> String {
        METHODS
            .iter()
            .map(|m| {
                if *m == self.request.method {
                    format!("[{m}]")
                } else {
                    m.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn body_format_label(&self) -> String {
        BODY_FORMATS
            .iter()
            .find(|(id, _, _)| *id == self.request.body_format)
            .map(|(_, label, _)| (*label).to_string())
            .unwrap_or_else(|| "Custom".to_string())
    }

    pub fn start_edit(&mut self) {
        self.input_mode = true;
        self.input_buffer = match self.editor_field {
            EditorField::Name => self.request.name.clone(),
            EditorField::Method => self.request.method.clone(),
            EditorField::Url => self.request.url.clone(),
            EditorField::Params => self.request.params_raw.clone(),
            EditorField::HeaderKey => self.header_key_input.clone(),
            EditorField::HeaderValue => self.header_value_input.clone(),
            EditorField::BodyFormat => self.request.body_format.clone(),
            EditorField::CustomContentType => self.request.custom_content_type.clone(),
            EditorField::Body => self.request.body.clone(),
            EditorField::Env => self.state.env_raw.clone(),
        };
        self.status = "Editing field — Ctrl+S save, Esc cancel".into();
    }

    pub fn apply_edit(&mut self) {
        match self.editor_field {
            EditorField::Name => self.request.name = self.input_buffer.clone(),
            EditorField::Method => self.request.method = self.input_buffer.to_uppercase(),
            EditorField::Url => self.request.url = self.input_buffer.clone(),
            EditorField::Params => self.request.params_raw = self.input_buffer.clone(),
            EditorField::HeaderKey => self.header_key_input = self.input_buffer.clone(),
            EditorField::HeaderValue => self.header_value_input = self.input_buffer.clone(),
            EditorField::BodyFormat => {
                self.request.body_format = self.input_buffer.clone();
                self.sync_content_type_header();
            }
            EditorField::CustomContentType => {
                self.request.custom_content_type = self.input_buffer.clone();
                self.sync_content_type_header();
            }
            EditorField::Body => self.request.body = self.input_buffer.clone(),
            EditorField::Env => self.state.env_raw = self.input_buffer.clone(),
        }
        self.input_mode = false;
        self.status = "Saved field".into();
    }

    pub fn send(&mut self) {
        if self.request.url.trim().is_empty() {
            self.status = "URL is required".into();
            return;
        }

        match execute_request(&self.request, &self.state.env_raw) {
            Ok(resp) => {
                self.status = format!(
                    "{} {} in {}ms",
                    resp.status_code, resp.reason, resp.elapsed_ms
                );
                self.response = Some(resp.clone());
                self.state.history.push(HistoryEntry {
                    timestamp: Utc::now(),
                    request: self.request.clone(),
                    response: resp,
                });
                if self.state.history.len() > 200 {
                    self.state.history.remove(0);
                }
                if let Err(e) = save_state(&self.state) {
                    self.status = format!("Saved request but failed persisting state: {e}");
                }
            }
            Err(err) => self.status = format!("Request failed: {err}"),
        }
    }

    pub fn save_collection(&mut self) {
        if self.request.name.trim().is_empty() {
            self.request.name = format!("Request {}", self.state.collections.len() + 1);
        }
        self.state
            .collections
            .retain(|r| r.name != self.request.name);
        self.state.collections.push(self.request.clone());
        self.collections_idx = self.state.collections.len().saturating_sub(1);
        match save_state(&self.state) {
            Ok(_) => self.status = format!("Saved collection: {}", self.request.name),
            Err(e) => self.status = format!("Save failed: {e}"),
        }
    }

    pub fn load_selected_collection(&mut self) {
        if let Some(item) = self.state.collections.get(self.collections_idx).cloned() {
            self.request = item;
            self.sync_content_type_header();
            self.status = "Collection loaded into editor".into();
        }
    }

    pub fn load_selected_history(&mut self) {
        if let Some(item) = self.state.history.get(self.history_idx).cloned() {
            self.request = item.request;
            self.response = Some(item.response);
            self.status = "Loaded request + response from history".into();
        }
    }

    pub fn new_request(&mut self) {
        self.request = ApiRequest::default();
        self.header_key_input.clear();
        self.header_value_input.clear();
        self.status = "New request".into();
    }

    pub fn curl_preview(&mut self) {
        let url = apply_query(
            &interpolate(&self.request.url, &parse_map(&self.state.env_raw)),
            &parse_map(&self.request.params_raw),
        );
        let mut cmd = format!("curl -X {} '{}'", self.request.method, url);
        for (k, v) in parse_map(&self.request.headers_raw) {
            cmd.push_str(&format!(" -H '{}: {}'", k, v));
        }
        if !self.request.body.trim().is_empty() {
            cmd.push_str(&format!(
                " --data '{}'",
                self.request.body.replace('\n', "\\n")
            ));
        }
        self.response = Some(ApiResponse {
            status_code: 0,
            reason: "cURL Preview".into(),
            elapsed_ms: 0,
            content_type: "text/plain".into(),
            headers: vec![],
            body: cmd,
        });
        self.status = "Generated cURL preview".into();
    }

    fn sync_content_type_header(&mut self) {
        let content_type = if self.request.body_format == "custom" {
            let custom = self.request.custom_content_type.trim();
            if custom.is_empty() {
                None
            } else {
                Some(custom.to_string())
            }
        } else {
            BODY_FORMATS
                .iter()
                .find(|(id, _, _)| *id == self.request.body_format)
                .and_then(|(_, _, content_type)| content_type.map(|ct| ct.to_string()))
        };

        let mut headers: Vec<(String, String)> = parse_map(&self.request.headers_raw)
            .into_iter()
            .filter(|(k, _)| !k.eq_ignore_ascii_case("Content-Type"))
            .collect();

        if let Some(ct) = content_type {
            headers.push(("Content-Type".into(), ct));
        }

        self.request.headers_raw = headers
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join("\n");
    }
}

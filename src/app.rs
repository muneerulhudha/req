use chrono::Utc;

use crate::{
    http_client::execute_request,
    model::{ApiRequest, ApiResponse, AppState, HistoryEntry},
    storage::{load_state, save_state},
    util::{apply_query, interpolate, parse_map},
};

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
    Headers,
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
            Params => Headers,
            Headers => Body,
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
            Headers => Params,
            Body => Headers,
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
        }
    }

    pub fn cycle_method(&mut self) {
        let methods = [
            "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "TRACE", "CONNECT",
        ];
        let pos = methods
            .iter()
            .position(|m| *m == self.request.method)
            .unwrap_or(0);
        self.request.method = methods[(pos + 1) % methods.len()].to_string();
    }

    pub fn start_edit(&mut self) {
        self.input_mode = true;
        self.input_buffer = match self.editor_field {
            EditorField::Name => self.request.name.clone(),
            EditorField::Method => self.request.method.clone(),
            EditorField::Url => self.request.url.clone(),
            EditorField::Params => self.request.params_raw.clone(),
            EditorField::Headers => self.request.headers_raw.clone(),
            EditorField::Body => self.request.body.clone(),
            EditorField::Env => self.state.env_raw.clone(),
        };
        self.status = "Editing field — Enter to save, Esc to cancel".into();
    }

    pub fn apply_edit(&mut self) {
        match self.editor_field {
            EditorField::Name => self.request.name = self.input_buffer.clone(),
            EditorField::Method => self.request.method = self.input_buffer.to_uppercase(),
            EditorField::Url => self.request.url = self.input_buffer.clone(),
            EditorField::Params => self.request.params_raw = self.input_buffer.clone(),
            EditorField::Headers => self.request.headers_raw = self.input_buffer.clone(),
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
}

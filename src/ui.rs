use chrono::Local;
use ratatui::{
    prelude::*,
    widgets::{block::Title, *},
};

use crate::app::{App, EditorField, Focus};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.size());

    let header = Paragraph::new("req — Terminal API Client | Tab focus | ↑/↓ navigate | E edit | M method | F body format | K header key | A add/update header | D delete header | S send | W save | L load | H load history | C cURL | N new | R toggle body/headers | Q quit")
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(header, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(22),
            Constraint::Percentage(43),
            Constraint::Percentage(35),
        ])
        .split(chunks[1]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(body[0]);

    let collection_items: Vec<ListItem> = app
        .state
        .collections
        .iter()
        .enumerate()
        .map(|(i, r)| ListItem::new(format!("{} {}. {}", r.method, i + 1, r.name)))
        .collect();

    let mut col_state = ListState::default();
    if !app.state.collections.is_empty() {
        col_state.select(Some(
            app.collections_idx.min(app.state.collections.len() - 1),
        ));
    }

    let col_block = Block::default()
        .title(Title::from("Collections").alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_style(if matches!(app.focus, Focus::Collections) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });
    frame.render_stateful_widget(
        List::new(collection_items)
            .highlight_symbol("▶ ")
            .block(col_block),
        left_chunks[0],
        &mut col_state,
    );

    let history_items: Vec<ListItem> = app
        .state
        .history
        .iter()
        .rev()
        .take(30)
        .enumerate()
        .map(|(i, h)| {
            let t = h.timestamp.with_timezone(&Local).format("%H:%M:%S");
            ListItem::new(format!(
                "{} {} {} {}",
                i + 1,
                t,
                h.response.status_code,
                h.request.url
            ))
        })
        .collect();

    let mut hist_state = ListState::default();
    if !history_items.is_empty() {
        hist_state.select(Some(app.history_idx.min(history_items.len() - 1)));
    }

    let hist_block = Block::default()
        .title(Title::from("History (latest 30)").alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_style(if matches!(app.focus, Focus::History) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });
    frame.render_stateful_widget(
        List::new(history_items)
            .highlight_symbol("▶ ")
            .block(hist_block),
        left_chunks[1],
        &mut hist_state,
    );

    render_request_editor(frame, app, body[1]);

    let response_title = if app.response_mode_headers {
        "Response Headers"
    } else {
        "Response Body"
    };
    let response_text = match (&app.response, app.response_mode_headers) {
        (Some(r), false) => format!(
            "Status: {} {} | {}ms | {}\n\n{}",
            r.status_code, r.reason, r.elapsed_ms, r.content_type, r.body
        ),
        (Some(r), true) => {
            let headers = r
                .headers
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join("\n");
            format!("Status: {} {}\n\n{}", r.status_code, r.reason, headers)
        }
        (None, _) => "No response yet. Press S to send request.".into(),
    };

    frame.render_widget(
        Paragraph::new(response_text)
            .wrap(Wrap { trim: false })
            .block(Block::default().title(response_title).borders(Borders::ALL)),
        body[2],
    );

    let status = if app.input_mode {
        format!("INPUT ({:?}): {}", app.editor_field, app.input_buffer)
    } else {
        app.status.clone()
    };
    frame.render_widget(
        Paragraph::new(status).block(Block::default().title("Status").borders(Borders::ALL)),
        chunks[2],
    );
}

fn render_request_editor(frame: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Min(10),
            Constraint::Length(4),
        ])
        .split(area);

    let top_lines = vec![
        line_for(
            "Name",
            &app.request.name,
            app.editor_field == EditorField::Name,
        ),
        line_for(
            "Method",
            &app.method_options(),
            app.editor_field == EditorField::Method,
        ),
        line_for(
            "URL",
            &app.request.url,
            app.editor_field == EditorField::Url,
        ),
        line_for(
            "Params",
            &app.request.params_raw,
            app.editor_field == EditorField::Params,
        ),
    ];

    frame.render_widget(
        Paragraph::new(top_lines).wrap(Wrap { trim: false }).block(
            Block::default()
                .title("Request Editor")
                .borders(Borders::ALL),
        ),
        rows[0],
    );

    let header_lines = vec![
        line_for(
            "Header Key",
            &app.header_key_input,
            app.editor_field == EditorField::HeaderKey,
        ),
        line_for(
            "Header Value",
            &app.header_value_input,
            app.editor_field == EditorField::HeaderValue,
        ),
        Line::from("  Tip: press K to cycle common headers, A to add/update, D to delete"),
        Line::from(""),
        Line::from(Span::styled(
            "  Active request headers:",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(app.request.headers_raw.clone()),
    ];

    frame.render_widget(
        Paragraph::new(header_lines)
            .wrap(Wrap { trim: false })
            .block(Block::default().title("Headers").borders(Borders::ALL)),
        rows[1],
    );

    let body_lines = vec![
        line_for(
            "Format",
            &app.body_format_label(),
            app.editor_field == EditorField::BodyFormat,
        ),
        line_for(
            "Custom Content-Type",
            &app.request.custom_content_type,
            app.editor_field == EditorField::CustomContentType,
        ),
        Line::from(Span::styled(
            "Body (multiline; press E to edit, Ctrl+S to save while editing):",
            Style::default().fg(Color::Cyan),
        )),
        Line::from(app.request.body.clone()),
    ];

    frame.render_widget(
        Paragraph::new(body_lines)
            .wrap(Wrap { trim: false })
            .block(Block::default().title("Body").borders(Borders::ALL)),
        rows[2],
    );

    frame.render_widget(
        Paragraph::new(line_for(
            "Env",
            &app.state.env_raw,
            app.editor_field == EditorField::Env,
        ))
        .wrap(Wrap { trim: false })
        .block(Block::default().title("Environment").borders(Borders::ALL)),
        rows[3],
    );
}

fn line_for(label: &str, value: &str, selected: bool) -> Line<'static> {
    let short = value.replace('\n', " ⏎ ");
    let prefix = if selected { "▶" } else { " " };
    Line::from(vec![
        Span::styled(
            format!("{prefix} {label}: "),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(short),
    ])
}

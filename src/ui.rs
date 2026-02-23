use chrono::Local;
use ratatui::{
    prelude::*,
    widgets::{block::Title, *},
};

use crate::app::{App, EditorField, Focus};

fn status_color(status_code: u16) -> Color {
    match status_code {
        200..=299 => Color::Green,
        300..=399 => Color::Blue,
        400..=499 => Color::Yellow,
        500..=599 => Color::Red,
        _ => Color::Gray,
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.size());

    let header = Paragraph::new("req — Terminal API Client | Tab focus | ↑/↓ navigate | PgUp/PgDn scroll response | E edit | M method | S send | W save | L load | H load history | C cURL | N new | R toggle body/headers | Q quit")
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(header, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(24),
            Constraint::Percentage(38),
            Constraint::Percentage(38),
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

    let editor_lines = vec![
        line_for(
            "Name",
            &app.request.name,
            app.editor_field == EditorField::Name,
        ),
        line_for(
            "Method",
            &app.request.method,
            app.editor_field == EditorField::Method,
        ),
        line_for(
            "URL",
            &app.request.url,
            app.editor_field == EditorField::Url,
        ),
        Line::from(""),
        line_for(
            "Params",
            &app.request.params_raw,
            app.editor_field == EditorField::Params,
        ),
        Line::from(""),
        line_for(
            "Headers",
            &app.request.headers_raw,
            app.editor_field == EditorField::Headers,
        ),
        Line::from(""),
        line_for(
            "Body",
            &app.request.body,
            app.editor_field == EditorField::Body,
        ),
        Line::from(""),
        line_for(
            "Env",
            &app.state.env_raw,
            app.editor_field == EditorField::Env,
        ),
    ];

    let edit_block = Block::default()
        .title("Request Editor")
        .borders(Borders::ALL)
        .border_style(if matches!(app.focus, Focus::Editor) {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        });
    frame.render_widget(
        Paragraph::new(editor_lines)
            .wrap(Wrap { trim: false })
            .block(edit_block),
        body[1],
    );

    let response_title = if app.response_mode_headers {
        "Response Headers"
    } else {
        "Response Body"
    };
    let (response_lines, response_block_style) = match (&app.response, app.response_mode_headers) {
        (Some(r), false) => {
            let color = status_color(r.status_code);
            let mut lines = vec![Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{} {}", r.status_code, r.reason),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" | {}ms | {}", r.elapsed_ms, r.content_type)),
            ])];
            lines.push(Line::from(""));
            lines.extend(r.body.lines().map(|line| Line::from(line.to_string())));
            (lines, Style::default().fg(color))
        }
        (Some(r), true) => {
            let color = status_color(r.status_code);
            let mut lines = vec![Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{} {}", r.status_code, r.reason),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
            ])];
            lines.push(Line::from(""));
            lines.extend(
                r.headers
                    .iter()
                    .map(|(k, v)| Line::from(format!("{}: {}", k, v))),
            );
            (lines, Style::default().fg(color))
        }
        (None, _) => (
            vec![Line::from("No response yet. Press S to send request.")],
            Style::default(),
        ),
    };

    frame.render_widget(
        Paragraph::new(response_lines)
            .scroll((app.response_scroll, 0))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .title(response_title)
                    .borders(Borders::ALL)
                    .border_style(response_block_style),
            ),
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

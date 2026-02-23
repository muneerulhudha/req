use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use req::{
    app::{App, Focus},
    ui,
};

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let result = (|| -> Result<()> {
        loop {
            terminal.draw(|f| ui::render(f, &app))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    if app.input_mode {
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && matches!(key.code, KeyCode::Char('s'))
                        {
                            app.apply_edit();
                            continue;
                        }

                        match key.code {
                            KeyCode::Esc => {
                                app.input_mode = false;
                                app.status = "Edit canceled".into();
                            }
                            KeyCode::Enter => app.input_buffer.push('\n'),
                            KeyCode::Backspace => {
                                app.input_buffer.pop();
                            }
                            KeyCode::Char(c) => app.input_buffer.push(c),
                            _ => {}
                        }
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Tab => {
                            app.focus = match app.focus {
                                Focus::Collections => Focus::History,
                                Focus::History => Focus::Editor,
                                Focus::Editor => Focus::Collections,
                            }
                        }
                        KeyCode::Up => match app.focus {
                            Focus::Collections => {
                                app.collections_idx = app.collections_idx.saturating_sub(1)
                            }
                            Focus::History => app.history_idx = app.history_idx.saturating_sub(1),
                            Focus::Editor => app.editor_field = app.editor_field.prev(),
                        },
                        KeyCode::Down => match app.focus {
                            Focus::Collections => {
                                app.collections_idx = (app.collections_idx + 1)
                                    .min(app.state.collections.len().saturating_sub(1))
                            }
                            Focus::History => {
                                app.history_idx = (app.history_idx + 1)
                                    .min(app.state.history.len().saturating_sub(1).min(29))
                            }
                            Focus::Editor => app.editor_field = app.editor_field.next(),
                        },
                        KeyCode::Char('e') => {
                            if matches!(app.focus, Focus::Editor) {
                                app.start_edit();
                            }
                        }
                        KeyCode::Char('m') => app.cycle_method(),
                        KeyCode::Char('f') => app.cycle_body_format(),
                        KeyCode::Char('k') => app.cycle_common_header(),
                        KeyCode::Char('a') => app.upsert_header(),
                        KeyCode::Char('d') => app.remove_header(),
                        KeyCode::Char('s') => app.send(),
                        KeyCode::Char('w') => app.save_collection(),
                        KeyCode::Char('l') => {
                            if matches!(app.focus, Focus::Collections) {
                                app.load_selected_collection();
                            }
                        }
                        KeyCode::Char('h') => {
                            if matches!(app.focus, Focus::History) {
                                app.load_selected_history();
                            }
                        }
                        KeyCode::Char('n') => app.new_request(),
                        KeyCode::Char('c') => app.curl_preview(),
                        KeyCode::Char('r') => {
                            app.response_mode_headers = !app.response_mode_headers
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    })();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

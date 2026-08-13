use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use crate::app::{App, Page};
use crate::config::expand_path;
use crate::ui::{box_and_hints, draw_hints, width, HIGHLIGHT};

// ---------------------------------------------------------------------------
// settings page
// ---------------------------------------------------------------------------

pub struct SettingsPage {
    pub input: String,
    pub saved: bool,
}

pub fn handle_settings(app: &mut App, key: KeyEvent) {
    if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
        app.go_main();
        return;
    }
    match key.code {
        KeyCode::Char(c) => {
            if let Page::Settings(p) = &mut app.page {
                p.input.push(c);
            }
        }
        KeyCode::Backspace => {
            if let Page::Settings(p) = &mut app.page {
                p.input.pop();
            }
        }
        KeyCode::Enter => {
            let input = if let Page::Settings(p) = &app.page {
                p.input.clone()
            } else {
                return;
            };
            app.config.origin_path_prefix = expand_path(&input, &app.home);
            app.save_config();
            if let Page::Settings(p) = &mut app.page {
                p.saved = true;
            }
        }
        _ => {}
    }
}

pub fn draw_settings(frame: &mut Frame, p: &SettingsPage) {
    let area = frame.area();
    // input + saved message + borders
    let (box_area, hints) = box_and_hints(area, 4);

    let block = Block::bordered().title("ConfCosmos - Settings");
    let inner = block.inner(box_area);
    frame.render_widget(block, box_area);

    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(inner);

    let label = format!("源路径前缀路径 (origin_path_prefix): {}", p.input);
    let label_w = width(&label);
    frame.render_widget(Paragraph::new(label).style(HIGHLIGHT), rows[0]);
    frame.set_cursor_position((rows[0].x + label_w, rows[0].y));

    if p.saved {
        frame.render_widget(
            Paragraph::new("已保存").style(Style::new().fg(Color::Green)),
            rows[1],
        );
    }

    draw_hints(frame, hints, "Enter 保存    Esc/q 返回");
}

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use crate::app::{App, Page};
use crate::config::expand_path;
use crate::ui::{box_and_hints, draw_hints, width, HIGHLIGHT};

// ---------------------------------------------------------------------------
// setup page (first run)
// ---------------------------------------------------------------------------

/// First-run page: ask for the source path prefix.
pub struct SetupPage {
    pub input: String,
}

pub fn handle_setup(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
        KeyCode::Char(c) => {
            if let Page::Setup(p) = &mut app.page {
                p.input.push(c);
            }
        }
        KeyCode::Backspace => {
            if let Page::Setup(p) = &mut app.page {
                p.input.pop();
            }
        }
        KeyCode::Enter => {
            let input = if let Page::Setup(p) = &app.page {
                p.input.clone()
            } else {
                return;
            };
            let expanded = expand_path(&input, &app.home);
            // mkdir -p on the parsed path so folders along the way exist
            if !expanded.is_empty() {
                let _ = std::fs::create_dir_all(&expanded);
            }
            app.config.origin_path_prefix = expanded;
            app.save_config();
            app.go_main();
        }
        _ => {}
    }
}

pub fn draw_setup(frame: &mut Frame, p: &SetupPage) {
    let area = frame.area();
    // input row + borders
    let (box_area, hints) = box_and_hints(area, 3);

    let block = Block::bordered().title("ConfCosmos - Setup");
    let inner = block.inner(box_area);
    frame.render_widget(block, box_area);

    let label = format!("源路径前缀路径 (origin_path_prefix): {}", p.input);
    let label_w = width(&label);
    frame.render_widget(Paragraph::new(label).style(HIGHLIGHT), inner);
    frame.set_cursor_position((inner.x + label_w, inner.y));

    draw_hints(
        frame,
        hints,
        "输入源路径前缀路径后按 Enter 确认    Esc 退出",
    );
}

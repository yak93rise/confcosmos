use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use crate::app::{App, Page};
use crate::pages::view_page::ViewPage;
use crate::ui::{box_and_hints, draw_hints, width, HIGHLIGHT};

// ---------------------------------------------------------------------------
// confirm dialog
// ---------------------------------------------------------------------------

pub enum ConfirmAction {
    /// Abandon the new-symbolic page and go back to the main page
    AbandonNew,
    /// Delete the given symbolic entry and go back to the view page
    DeleteEntry(String),
}

pub struct ConfirmPage {
    pub message: String,
    pub action: ConfirmAction,
    /// Selected operation button (0 = 确认, 1 = 取消)
    pub selected: usize,
}

enum ConfirmChoice {
    Yes,
    No,
    None,
}

pub fn handle_confirm(app: &mut App, key: KeyEvent) {
    let choice = if let Page::Confirm(p) = &mut app.page {
        match key.code {
            // ←/→ move between the 确认 / 取消 buttons
            KeyCode::Left | KeyCode::Right => {
                p.selected = (p.selected + 1) % 2;
                ConfirmChoice::None
            }
            // Enter executes the selected button
            KeyCode::Enter => {
                if p.selected == 0 {
                    ConfirmChoice::Yes
                } else {
                    ConfirmChoice::No
                }
            }
            // shortcuts: y confirms, n/Esc cancels
            KeyCode::Char('y') | KeyCode::Char('Y') => ConfirmChoice::Yes,
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => ConfirmChoice::No,
            _ => ConfirmChoice::None,
        }
    } else {
        return;
    };

    match choice {
        ConfirmChoice::Yes => {
            let action = match &app.page {
                Page::Confirm(p) => &p.action,
                _ => return,
            };
            match action {
                ConfirmAction::AbandonNew => {
                    app.prev = None;
                    app.go_main();
                }
                ConfirmAction::DeleteEntry(name) => {
                    let name = name.clone();
                    app.config.symbolic.remove(&name);
                    app.save_config();
                    let mut vp = ViewPage::from_config(&app.config);
                    vp.selected = 0;
                    app.page = Page::View(vp);
                    app.prev = None;
                }
            }
        }
        ConfirmChoice::No => {
            if let Some(prev) = app.prev.take() {
                app.page = prev;
            } else {
                app.go_main();
            }
        }
        ConfirmChoice::None => {}
    }
}

pub fn draw_confirm(frame: &mut Frame, p: &ConfirmPage) {
    let area = frame.area();
    // message + buttons + borders
    let (box_area, hints) = box_and_hints(area, 4);

    let block = Block::bordered().title("ConfCosmos - Confirm");
    let inner = block.inner(box_area);
    frame.render_widget(block, box_area);

    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(inner);

    let w = width(&p.message);
    let h = Layout::horizontal([Constraint::Length(w)])
        .flex(Flex::Center)
        .split(rows[0]);
    frame.render_widget(Paragraph::new(p.message.as_str()), h[0]);

    let mut spans: Vec<Span> = Vec::new();
    for (i, label) in ["确认", "取消"].iter().enumerate() {
        let focused = p.selected == i;
        let style = if focused {
            HIGHLIGHT.add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        spans.push(Span::styled(format!("[ {} ]", label), style));
        if i == 0 {
            spans.push(Span::raw("    "));
        }
    }
    let bw = width("[ 确认 ]    [ 取消 ]");
    let bh = Layout::horizontal([Constraint::Length(bw)])
        .flex(Flex::Center)
        .split(rows[1]);
    frame.render_widget(Paragraph::new(Line::from(spans)), bh[0]);

    draw_hints(frame, hints, "←/→ 选择按钮    Enter 执行    Esc 取消");
}

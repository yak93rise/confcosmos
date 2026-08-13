use crossterm::event::{KeyCode, KeyEvent};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::{App, Page};
use crate::config::{Config, SymbolicEntry};
use crate::pages::manage_page::ManagePage;
use crate::ui::{box_and_hints, draw_hints, HIGHLIGHT};

// ---------------------------------------------------------------------------
// symbolic_view_page
// ---------------------------------------------------------------------------

/// `symbolic_view_page` state.
pub struct ViewPage {
    pub entries: Vec<(String, SymbolicEntry)>,
    pub selected: usize,
}

impl ViewPage {
    pub fn from_config(config: &Config) -> Self {
        let entries: Vec<(String, SymbolicEntry)> = config
            .symbolic
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Self {
            entries,
            selected: 0,
        }
    }
}

pub fn handle_view(app: &mut App, key: KeyEvent) {
    if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
        app.go_main();
        return;
    }
    if key.code == KeyCode::Enter {
        // Enter on a list item opens the manage page for that entry
        let manage = match &app.page {
            Page::View(v) => v
                .entries
                .get(v.selected)
                .map(|(n, e)| ManagePage::new(n.clone(), e.clone(), v.selected)),
            _ => None,
        };
        if let Some(m) = manage {
            app.page = Page::Manage(m);
        }
        return;
    }
    if let Page::View(v) = &mut app.page {
        match key.code {
            KeyCode::Up => v.selected = v.selected.saturating_sub(1),
            KeyCode::Down => v.selected = (v.selected + 1).min(v.entries.len().saturating_sub(1)),
            _ => {}
        }
    }
}

pub fn draw_view(frame: &mut Frame, p: &ViewPage) {
    let area = frame.area();
    // every entry is one line; the focused entry adds two detail lines, so
    // the total height stays constant while moving the selection
    let box_h = (p.entries.len() as u16 + 2 + 2).max(3);
    let (box_area, hints) = box_and_hints(area, box_h);

    let block = Block::bordered().title("ConfCosmos - View");
    let inner = block.inner(box_area);
    frame.render_widget(block, box_area);

    if p.entries.is_empty() {
        frame.render_widget(
            Paragraph::new("暂无软链接配置，请先在主菜单选择「新增软链接」")
                .style(Style::new().fg(Color::DarkGray)),
            inner,
        );
    } else {
        let items: Vec<ListItem> = p
            .entries
            .iter()
            .enumerate()
            .map(|(i, (name, entry))| {
                if i == p.selected {
                    ListItem::new(vec![
                        Line::from(format!("{}. {}", i + 1, name)),
                        Line::from(format!("   origin_path: {}", entry.origin_path)),
                        Line::from(format!("   target_path: {}", entry.target_path)),
                    ])
                } else {
                    ListItem::new(Line::from(format!("{}. {}", i + 1, name)))
                }
            })
            .collect();
        let list = List::new(items)
            .highlight_style(HIGHLIGHT)
            .highlight_symbol("> ");
        let mut state = ListState::default();
        state.select(Some(p.selected));
        frame.render_stateful_widget(list, inner, &mut state);
    }

    draw_hints(frame, hints, "↑/↓ 选择项目    Enter 管理    Esc/q 返回");
}

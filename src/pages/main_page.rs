use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::style::Modifier;
use ratatui::widgets::{Block, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::{App, Page, MENU_ITEMS};
use crate::pages::generate_page::GeneratePage;
use crate::pages::new_page::NewPage;
use crate::pages::settings_page::SettingsPage;
use crate::pages::view_page::ViewPage;
use crate::ui::{box_and_hints, draw_hints, width, HIGHLIGHT};

// ---------------------------------------------------------------------------
// main page
// ---------------------------------------------------------------------------

pub fn handle_main(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up => app.menu_selected = app.menu_selected.saturating_sub(1),
        KeyCode::Down => app.menu_selected = (app.menu_selected + 1).min(MENU_ITEMS.len() - 1),
        KeyCode::Enter => match app.menu_selected {
            0 => app.page = Page::View(ViewPage::from_config(&app.config)),
            1 => app.page = Page::New(NewPage::new()),
            2 => app.page = Page::Generate(GeneratePage::from_config(&app.config)),
            _ => {
                app.page = Page::Settings(SettingsPage {
                    input: app.config.origin_path_prefix.clone(),
                    saved: false,
                })
            }
        },
        KeyCode::Char('q') => app.quit = true,
        _ => {}
    }
}

pub fn draw_main(frame: &mut Frame, app: &App) {
    let area = frame.area();
    // menu items + borders
    let (box_area, hints) = box_and_hints(area, MENU_ITEMS.len() as u16 + 2);

    let block = Block::bordered().title("ConfCosmos - Main");
    let inner = block.inner(box_area);
    frame.render_widget(block, box_area);

    // Horizontally centered menu inside the box
    let max_w = MENU_ITEMS.iter().map(|l| width(l)).max().unwrap_or(0) + 2 + 4;
    let h = Layout::horizontal([Constraint::Length(max_w)])
        .flex(Flex::Center)
        .split(inner);

    let items: Vec<ListItem> = MENU_ITEMS
        .iter()
        .map(|m| ListItem::new(m.to_string()))
        .collect();
    let list = List::new(items)
        .highlight_style(HIGHLIGHT.add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    let mut state = ListState::default();
    state.select(Some(app.menu_selected));
    frame.render_stateful_widget(list, h[0], &mut state);

    draw_hints(frame, hints, "↑/↓ 选择菜单项    Enter 确认    q 退出工具");
}

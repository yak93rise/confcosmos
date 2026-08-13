use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, Page};
use crate::pages;

/// 高亮样式（各页面共用）。
pub(crate) const HIGHLIGHT: Style = Style::new().bg(Color::Blue).fg(Color::White);

/// Display width in terminal columns (CJK chars are double width).
pub(crate) fn width(s: &str) -> u16 {
    s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

/// 渲染当前激活的页面。每个页面模块拥有自己的 `draw_<name>` 函数。
pub fn draw(frame: &mut Frame, app: &App) {
    match &app.page {
        Page::Main => pages::main_page::draw_main(frame, app),
        Page::Setup(p) => pages::setup_page::draw_setup(frame, p),
        Page::New(p) => pages::new_page::draw_new(frame, p),
        Page::View(p) => pages::view_page::draw_view(frame, p),
        Page::Manage(p) => pages::manage_page::draw_manage(frame, p),
        Page::Generate(p) => pages::generate_page::draw_generate(frame, p),
        Page::Settings(p) => pages::settings_page::draw_settings(frame, p),
        Page::Confirm(p) => pages::confirm_page::draw_confirm(frame, p),
    }
}

/// The bordered box hugs the page content: its height is `box_h` (the content
/// plus the two borders) instead of the whole window. The box and the hint
/// line below its bottom border are vertically centered in the window.
pub(crate) fn box_and_hints(area: Rect, box_h: u16) -> (Rect, Rect) {
    let max_h = area.height.saturating_sub(1).max(3);
    let box_h = box_h.clamp(3, max_h);
    let group = Layout::vertical([Constraint::Length(box_h), Constraint::Length(1)])
        .flex(Flex::Center)
        .split(area);
    (group[0], group[1])
}

pub(crate) fn draw_hints(frame: &mut Frame, area: Rect, hints: &str) {
    frame.render_widget(
        Paragraph::new(hints).style(Style::new().fg(Color::DarkGray)),
        area,
    );
}

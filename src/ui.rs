use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::{
    GenMode, GenStatus, MENU_ITEMS, App, ConfirmPage, GeneratePage, ManagePage, NewFocus,
    NewPage, Page, SettingsPage, SetupPage, ViewPage,
};

const HIGHLIGHT: Style = Style::new().bg(Color::Blue).fg(Color::White);

/// Display width in terminal columns (CJK chars are double width).
fn width(s: &str) -> u16 {
    s.chars().map(|c| if c.is_ascii() { 1 } else { 2 }).sum()
}

pub fn draw(frame: &mut Frame, app: &App) {
    match &app.page {
        Page::Main => draw_main(frame, app),
        Page::Setup(p) => draw_setup(frame, p),
        Page::New(p) => draw_new(frame, p),
        Page::View(p) => draw_view(frame, p),
        Page::Manage(p) => draw_manage(frame, p),
        Page::Generate(p) => draw_generate(frame, p),
        Page::Settings(p) => draw_settings(frame, p),
        Page::Confirm(p) => draw_confirm(frame, p),
    }
}

/// The bordered box hugs the page content: its height is `box_h` (the content
/// plus the two borders) instead of the whole window. The box and the hint
/// line below its bottom border are vertically centered in the window.
fn box_and_hints(area: Rect, box_h: u16) -> (Rect, Rect) {
    let max_h = area.height.saturating_sub(1).max(3);
    let box_h = box_h.clamp(3, max_h);
    let group = Layout::vertical([Constraint::Length(box_h), Constraint::Length(1)])
        .flex(Flex::Center)
        .split(area);
    (group[0], group[1])
}

fn draw_hints(frame: &mut Frame, area: Rect, hints: &str) {
    frame.render_widget(
        Paragraph::new(hints).style(Style::new().fg(Color::DarkGray)),
        area,
    );
}

// ---------------------------------------------------------------------------
// main_page
// ---------------------------------------------------------------------------

fn draw_main(frame: &mut Frame, app: &App) {
    let area = frame.area();
    // menu items + borders
    let (box_area, hints) = box_and_hints(area, MENU_ITEMS.len() as u16 + 2);

    let block = Block::bordered().title("ConfCosmos - Main");
    let inner = block.inner(box_area);
    frame.render_widget(block, box_area);

    // Horizontally centered menu inside the box
    let max_w = MENU_ITEMS.iter().map(|l| width(l)).max().unwrap_or(0) + 2 + 4;
    let h = Layout::horizontal([Constraint::Length(max_w)]).flex(Flex::Center).split(inner);

    let items: Vec<ListItem> = MENU_ITEMS.iter().map(|m| ListItem::new(m.to_string())).collect();
    let list = List::new(items)
        .highlight_style(HIGHLIGHT.add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    let mut state = ListState::default();
    state.select(Some(app.menu_selected));
    frame.render_stateful_widget(list, h[0], &mut state);

    draw_hints(frame, hints, "↑/↓ 选择菜单项    Enter 确认    q 退出工具");
}

// ---------------------------------------------------------------------------
// setup page (first run)
// ---------------------------------------------------------------------------

fn draw_setup(frame: &mut Frame, p: &SetupPage) {
    let area = frame.area();
    // input row + borders
    let (box_area, hints) = box_and_hints(area, 3);

    let block = Block::bordered().title("ConfCosmos - Setup");
    let inner = block.inner(box_area);
    frame.render_widget(block, box_area);

    let label = format!("源路径前缀路径 (origin_path_prefix): {}", p.input);
    let label_w = width(&label);
    frame.render_widget(
        Paragraph::new(label).style(HIGHLIGHT),
        inner,
    );
    frame.set_cursor_position((inner.x + label_w, inner.y));

    draw_hints(frame, hints, "输入源路径前缀路径后按 Enter 确认    Esc 退出");
}

// ---------------------------------------------------------------------------
// symbolic_new_page
// ---------------------------------------------------------------------------

fn draw_new(frame: &mut Frame, p: &NewPage) {
    let area = frame.area();
    // 3 labels + 3 input rows + 2 gaps + notice + buttons + gap + edit hint + borders
    let (box_area, hints) = box_and_hints(area, 15);

    let block = Block::bordered().title("ConfCosmos - New");
    let inner = block.inner(box_area);
    frame.render_widget(block, box_area);

    let rows = Layout::vertical([
        Constraint::Length(1), // 软链接名称 label
        Constraint::Length(1), // input
        Constraint::Length(1), // gap
        Constraint::Length(1), // 源路径 label
        Constraint::Length(1), // input
        Constraint::Length(1), // gap
        Constraint::Length(1), // 目标路径 label
        Constraint::Length(1), // input
        Constraint::Length(1), // gap
        Constraint::Length(1), // notice
        Constraint::Length(1), // buttons
        Constraint::Length(1), // gap between buttons and hint
        Constraint::Length(1), // edit hint
    ])
    .split(inner);
    let notice_area = rows[9];
    let buttons_area = rows[10];
    let edit_hint_area = rows[12];

    let mut cursor: Option<(u16, u16)> = None;

    // Fields appear one by one (name -> origin -> target)
    let visible_fields = p.fixed.min(2) + 1;
    for i in 0..visible_fields {
        let label_row = rows[3 * i as usize];
        let input_row = rows[3 * i as usize + 1];

        let (label, value, is_active, is_selected, editing_this) = match i {
            0 => (
                "软链接名称",
                &p.symbolic_name,
                p.fixed == 0 && p.focus == NewFocus::Input,
                p.fixed == 3 && p.focus == NewFocus::Input && p.selected == 0,
                p.fixed == 3 && p.focus == NewFocus::Input && p.selected == 0 && p.editing,
            ),
            1 => (
                "源路径",
                &p.origin_path,
                p.fixed == 1 && p.focus == NewFocus::Input,
                p.fixed == 3 && p.focus == NewFocus::Input && p.selected == 1,
                p.fixed == 3 && p.focus == NewFocus::Input && p.selected == 1 && p.editing,
            ),
            _ => (
                "目标路径",
                &p.target_path,
                p.fixed == 2 && p.focus == NewFocus::Input,
                p.fixed == 3 && p.focus == NewFocus::Input && p.selected == 2,
                p.fixed == 3 && p.focus == NewFocus::Input && p.selected == 2 && p.editing,
            ),
        };

        // label on its own line, horizontally centered
        let lw = width(label);
        let lh = Layout::horizontal([Constraint::Length(lw)])
            .flex(Flex::Center)
            .split(label_row);
        frame.render_widget(Paragraph::new(Span::raw(label)), lh[0]);

        // input on the next line; the background grows with the typed text
        // and the text itself is horizontally centered
        let box_style = if is_active || editing_this {
            Style::new().fg(Color::Black).bg(Color::Blue)
        } else if is_selected {
            Style::new().fg(Color::Black).bg(Color::DarkGray)
        } else {
            Style::default()
        };
        let value_w = width(value);
        let left_pad = input_row.width.saturating_sub(value_w) / 2;
        let line = Line::from(vec![
            Span::raw(" ".repeat(left_pad as usize)),
            Span::styled(value.as_str(), box_style),
        ]);
        frame.render_widget(Paragraph::new(line), input_row);
        if is_active || editing_this {
            cursor = Some((input_row.x + left_pad + value_w, input_row.y));
        }
    }

    if let Some(msg) = &p.notice {
        let w = width(msg);
        let h = Layout::horizontal([Constraint::Length(w)])
            .flex(Flex::Center)
            .split(notice_area);
        frame.render_widget(
            Paragraph::new(msg.as_str()).style(Style::new().fg(Color::Red)),
            h[0],
        );
    }

    if p.fixed == 3 {
        let mut spans: Vec<Span> = Vec::new();
        for (i, label) in ["确认新增", "返回修改", "放弃新增"].iter().enumerate() {
            let focused = p.focus == NewFocus::Buttons && p.button_idx == i;
            let style = if focused {
                HIGHLIGHT.add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            spans.push(Span::styled(format!("[ {} ]", label), style));
            if i < 2 {
                spans.push(Span::raw("    "));
            }
        }
        let bw = width("[ 确认新增 ]    [ 返回修改 ]    [ 放弃新增 ]");
        let bh = Layout::horizontal([Constraint::Length(bw)])
            .flex(Flex::Center)
            .split(buttons_area);
        frame.render_widget(Paragraph::new(Line::from(spans)), bh[0]);

        // edit hint below the operation buttons
        let hint = "如果输入有误可按下 Tab 后选择有误内容按下 Enter 进行编辑";
        let hw = width(hint);
        let hh = Layout::horizontal([Constraint::Length(hw)])
            .flex(Flex::Center)
            .split(edit_hint_area);
        frame.render_widget(
            Paragraph::new(hint).style(Style::new().fg(Color::DarkGray)),
            hh[0],
        );
    }

    if let Some((x, y)) = cursor {
        frame.set_cursor_position((x, y));
    }

    draw_hints(
        frame,
        hints,
        "↑/↓ 选择项目    ←/→ 选择操作按钮    Enter 确认    Tab 切换输入区域和操作按钮    Esc 放弃并返回",
    );
}

// ---------------------------------------------------------------------------
// symbolic_view_page
// ---------------------------------------------------------------------------

fn draw_view(frame: &mut Frame, p: &ViewPage) {
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

// ---------------------------------------------------------------------------
// symbolic_manage_page
// ---------------------------------------------------------------------------

fn draw_manage(frame: &mut Frame, p: &ManagePage) {
    let area = frame.area();
    // name + origin + target + datetime + buttons + borders
    let (box_area, hints) = box_and_hints(area, 8);

    let block = Block::bordered().title("ConfCosmos - Manage");
    let inner = block.inner(box_area);
    frame.render_widget(block, box_area);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner);
    let buttons_area = rows[rows.len() - 1];

    let mut cursor: Option<(u16, u16)> = None;

    let fields: [(&str, &String); 3] = [
        ("软链接名称: ", &p.symbolic_name),
        ("源路径: ", &p.origin_path),
        ("目标路径: ", &p.target_path),
    ];
    for (i, (label_text, value)) in fields.iter().enumerate() {
        let selected_row = p.focus == NewFocus::Input && p.selected == i;
        let editing_row = selected_row && p.editing;

        let label_w = width(label_text);
        frame.render_widget(Paragraph::new(Span::raw(*label_text)), rows[i]);
        let box_area_row = Rect {
            x: rows[i].x + label_w,
            y: rows[i].y,
            width: rows[i].width.saturating_sub(label_w),
            height: 1,
        };
        let box_style = if editing_row {
            Style::new().fg(Color::Black).bg(Color::Blue)
        } else if selected_row {
            Style::new().fg(Color::Black).bg(Color::DarkGray)
        } else {
            Style::default()
        };
        frame.render_widget(Paragraph::new(value.as_str()).style(box_style), box_area_row);
        if editing_row {
            cursor = Some((box_area_row.x + width(value), rows[i].y));
        }
    }

    // read-only datetime row
    let dt_label = format!("新增时间: {}", p.add_datetime);
    frame.render_widget(Paragraph::new(Span::raw(dt_label)), rows[3]);

    // operation buttons
    let mut spans: Vec<Span> = Vec::new();
    for (i, label) in ["修改", "删除", "返回"].iter().enumerate() {
        let focused = p.focus == NewFocus::Buttons && p.button_idx == i;
        let style = if focused {
            HIGHLIGHT.add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        spans.push(Span::styled(format!("[ {} ]", label), style));
        if i < 2 {
            spans.push(Span::raw("    "));
        }
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), buttons_area);

    if let Some((x, y)) = cursor {
        frame.set_cursor_position((x, y));
    }

    draw_hints(
        frame,
        hints,
        "↑/↓ 选择项目    ←/→ 选择操作按钮    Enter 确认    Tab 切换输入区域和操作按钮    Esc 返回",
    );
}

// ---------------------------------------------------------------------------
// generate page
// ---------------------------------------------------------------------------

fn draw_generate(frame: &mut Frame, p: &GeneratePage) {
    let area = frame.area();
    let box_h = match p.mode {
        // list + notice + buttons + borders
        GenMode::Select => p.entries.len() as u16 + 4,
        // results + borders
        GenMode::Results => p.results.len() as u16 + 2,
    };
    let (box_area, hints) = box_and_hints(area, box_h.max(3));

    let block = Block::bordered().title("ConfCosmos - Generate");
    let inner = block.inner(box_area);
    frame.render_widget(block, box_area);

    match p.mode {
        GenMode::Select => {
            if p.entries.is_empty() {
                frame.render_widget(
                    Paragraph::new("暂无软链接配置，请先在主菜单选择「新增软链接」")
                        .style(Style::new().fg(Color::DarkGray)),
                    inner,
                );
                draw_hints(frame, hints, "Esc/q 返回");
            } else {
                let rows = Layout::vertical([
                    Constraint::Min(0),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(inner);

                let items: Vec<ListItem> = p
                    .entries
                    .iter()
                    .enumerate()
                    .map(|(i, (name, _))| {
                        let mark = if p.checks[i] { "[x]" } else { "[ ]" };
                        ListItem::new(Line::from(format!("{mark} {}. {name}", i + 1)))
                    })
                    .collect();
                let list = List::new(items)
                    .highlight_style(HIGHLIGHT)
                    .highlight_symbol("> ");
                let mut state = ListState::default();
                state.select(Some(p.cursor));
                frame.render_stateful_widget(list, rows[0], &mut state);

                if let Some(msg) = &p.notice {
                    frame.render_widget(
                        Paragraph::new(msg.as_str()).style(Style::new().fg(Color::Red)),
                        rows[1],
                    );
                }

                let mut spans: Vec<Span> = Vec::new();
                for (i, label) in ["确认生成", "返回"].iter().enumerate() {
                    let focused = p.focus == NewFocus::Buttons && p.button_idx == i;
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
                frame.render_widget(Paragraph::new(Line::from(spans)), rows[2]);

                draw_hints(
                    frame,
                    hints,
                    "↑/↓ 选择项目    Space/Enter 勾选    Tab 切换列表和操作按钮    Esc/q 返回",
                );
            }
        }
        GenMode::Results => {
            if p.results.is_empty() {
                frame.render_widget(
                    Paragraph::new("没有生成任何软链接").style(Style::new().fg(Color::DarkGray)),
                    inner,
                );
            } else {
                let items: Vec<ListItem> = p
                    .results
                    .iter()
                    .map(|(name, status)| {
                        let (line, style) = match status {
                            GenStatus::Success => (format!("{name} : 已创建软链接"), Color::Green),
                            GenStatus::Exists => (format!("{name} : 目标已存在，跳过"), Color::Yellow),
                            GenStatus::Failed(e) => (format!("{name} : 失败 - {e}"), Color::Red),
                        };
                        ListItem::new(Line::from(Span::styled(line, Style::new().fg(style))))
                    })
                    .collect();
                frame.render_widget(List::new(items), inner);
            }
            draw_hints(frame, hints, "Enter 重新选择    Esc/q 返回");
        }
    }
}

// ---------------------------------------------------------------------------
// settings page
// ---------------------------------------------------------------------------

fn draw_settings(frame: &mut Frame, p: &SettingsPage) {
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

// ---------------------------------------------------------------------------
// confirm dialog
// ---------------------------------------------------------------------------

fn draw_confirm(frame: &mut Frame, p: &ConfirmPage) {
    let area = frame.area();
    // message + buttons + borders
    let (box_area, hints) = box_and_hints(area, 4);

    let block = Block::bordered().title("ConfCosmos - Confirm");
    let inner = block.inner(box_area);
    frame.render_widget(block, box_area);

    let rows = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(inner);

    let w = width(&p.message);
    let h = Layout::horizontal([Constraint::Length(w)]).flex(Flex::Center).split(rows[0]);
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
    let bh = Layout::horizontal([Constraint::Length(bw)]).flex(Flex::Center).split(rows[1]);
    frame.render_widget(Paragraph::new(Line::from(spans)), bh[0]);

    draw_hints(frame, hints, "←/→ 选择按钮    Enter 执行    Esc 取消");
}

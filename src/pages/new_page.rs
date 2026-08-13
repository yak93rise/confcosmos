use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use crate::app::{App, Page};
use crate::config::{expand_path, join_origin_path, SymbolicEntry};
use crate::pages::confirm_page::ConfirmAction;
use crate::pages::NewFocus;
use crate::ui::{box_and_hints, draw_hints, width, HIGHLIGHT};

// ---------------------------------------------------------------------------
// symbolic_new_page
// ---------------------------------------------------------------------------

/// `symbolic_new_page` state.
pub struct NewPage {
    pub symbolic_name: String,
    pub origin_path: String,
    pub target_path: String,
    /// How many inputs are already fixed (0..=3). Once 3, the buttons appear.
    pub fixed: usize,
    pub focus: NewFocus,
    /// Selected input item while editing (0 = symbolic_name, 1 = origin_path, 2 = target_path)
    pub selected: usize,
    /// Whether the selected item is being edited inline
    pub editing: bool,
    /// Selected operation button (0 = 确认新增, 1 = 返回修改, 2 = 放弃新增)
    pub button_idx: usize,
    /// Transient message, e.g. "内容不能为空，若放弃新增可按 Esc 键返回主页"
    pub notice: Option<String>,
}

pub(crate) enum NewResult {
    None,
    Abandon,
    ConfirmAbandon,
    Save,
}

impl NewPage {
    pub fn new() -> Self {
        Self {
            symbolic_name: String::new(),
            origin_path: String::new(),
            target_path: String::new(),
            fixed: 0,
            focus: NewFocus::Input,
            selected: 0,
            editing: false,
            button_idx: 0,
            notice: None,
        }
    }

    pub(crate) fn handle(&mut self, key: KeyEvent) -> NewResult {
        // Esc: abandon the new-symbolic page and go back to the main page directly
        if key.code == KeyCode::Esc {
            return NewResult::Abandon;
        }
        // Any interaction clears the previous notice
        self.notice = None;

        match self.focus {
            NewFocus::Buttons => {
                match key.code {
                    KeyCode::Left => self.button_idx = (self.button_idx + 2) % 3,
                    KeyCode::Right => self.button_idx = (self.button_idx + 1) % 3,
                    // Tab switches between the input area and the buttons
                    KeyCode::Tab => {
                        self.focus = NewFocus::Input;
                        self.selected = 0;
                        self.editing = false;
                    }
                    KeyCode::Enter => {
                        return match self.button_idx {
                            0 => {
                                // 确认新增: all three inputs must be non-empty
                                if self.symbolic_name.is_empty()
                                    || self.origin_path.is_empty()
                                    || self.target_path.is_empty()
                                {
                                    self.notice = Some(
                                        "内容不能为空，若放弃新增可按 Esc 键返回主页".to_string(),
                                    );
                                    NewResult::None
                                } else {
                                    NewResult::Save
                                }
                            }
                            1 => {
                                // 返回修改: focus the symbolic_name and allow
                                // up/down selection over the three items.
                                self.focus = NewFocus::Input;
                                self.selected = 0;
                                self.editing = false;
                                NewResult::None
                            }
                            _ => NewResult::ConfirmAbandon,
                        };
                    }
                    _ => {}
                }
                NewResult::None
            }
            NewFocus::Input => {
                if self.fixed < 3 {
                    // Sequential first pass: name -> origin -> target
                    let buf = match self.fixed {
                        0 => &mut self.symbolic_name,
                        1 => &mut self.origin_path,
                        _ => &mut self.target_path,
                    };
                    match key.code {
                        KeyCode::Char(c) => buf.push(c),
                        KeyCode::Backspace => {
                            buf.pop();
                        }
                        KeyCode::Enter => {
                            // empty input is not allowed
                            if buf.is_empty() {
                                self.notice =
                                    Some("内容不能为空，若放弃新增可按 Esc 键返回主页".to_string());
                            } else {
                                self.fixed += 1;
                                if self.fixed == 3 {
                                    // After the target path is confirmed, move
                                    // the focus to the 确认新增 button; Tab
                                    // switches back to the input area for
                                    // editing, and Enter in edit state saves
                                    // while keeping the focus in the input.
                                    self.focus = NewFocus::Buttons;
                                    self.button_idx = 0;
                                }
                            }
                        }
                        _ => {}
                    }
                } else if self.editing {
                    // Inline editing of the selected item (返回修改 mode)
                    let buf = match self.selected {
                        0 => &mut self.symbolic_name,
                        1 => &mut self.origin_path,
                        _ => &mut self.target_path,
                    };
                    match key.code {
                        KeyCode::Char(c) => buf.push(c),
                        KeyCode::Backspace => {
                            buf.pop();
                        }
                        KeyCode::Enter => {
                            // empty input is not allowed here either
                            if buf.is_empty() {
                                self.notice =
                                    Some("内容不能为空，若放弃新增可按 Esc 键返回主页".to_string());
                            } else {
                                self.editing = false;
                                // The focus stays in the input area after
                                // saving; only Tab switches to the buttons.
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Selection among the three items
                    match key.code {
                        KeyCode::Up => self.selected = self.selected.saturating_sub(1),
                        KeyCode::Down => self.selected = (self.selected + 1).min(2),
                        KeyCode::Enter => self.editing = true,
                        KeyCode::Tab => self.focus = NewFocus::Buttons,
                        _ => {}
                    }
                }
                NewResult::None
            }
        }
    }
}

pub fn handle_new(app: &mut App, key: KeyEvent) {
    let action = if let Page::New(p) = &mut app.page {
        p.handle(key)
    } else {
        return;
    };
    match action {
        NewResult::None => {}
        NewResult::Abandon => app.go_main(),
        NewResult::ConfirmAbandon => {
            app.show_confirm(
                "确认放弃新增并返回主页面？".to_string(),
                ConfirmAction::AbandonNew,
            );
        }
        NewResult::Save => {
            let entry = SymbolicEntry {
                origin_path: join_origin_path(&app.config.origin_path_prefix, p_origin(app)),
                target_path: expand_path(p_target(app), &app.home),
                add_datetime: chrono::Local::now()
                    .format("%Y-%m-%d %H:%M:%S %3f")
                    .to_string(),
            };
            app.config.symbolic.insert(p_name(app).to_string(), entry);
            app.save_config();
            app.go_main();
        }
    }
}

// Small helpers to read the New page state while `app` is not mutably borrowed.
fn p_name(app: &App) -> &str {
    if let Page::New(p) = &app.page {
        &p.symbolic_name
    } else {
        ""
    }
}
fn p_origin(app: &App) -> &str {
    if let Page::New(p) = &app.page {
        &p.origin_path
    } else {
        ""
    }
}
fn p_target(app: &App) -> &str {
    if let Page::New(p) = &app.page {
        &p.target_path
    } else {
        ""
    }
}

pub fn draw_new(frame: &mut Frame, p: &NewPage) {
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
        let label_row = rows[3 * i];
        let input_row = rows[3 * i + 1];

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

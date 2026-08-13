use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use crate::app::{App, Page};
use crate::config::{expand_path, SymbolicEntry};
use crate::pages::confirm_page::ConfirmAction;
use crate::pages::view_page::ViewPage;
use crate::pages::NewFocus;
use crate::ui::{box_and_hints, draw_hints, width, HIGHLIGHT};

// ---------------------------------------------------------------------------
// symbolic_manage_page
// ---------------------------------------------------------------------------

/// `symbolic_manage_page` state: modify / delete / return for one entry.
pub struct ManagePage {
    /// Config key at page-open time (used to remove the old entry on rename)
    pub key: String,
    pub symbolic_name: String,
    pub origin_path: String,
    pub target_path: String,
    pub add_datetime: String,
    /// View list index to restore when returning
    pub back_selected: usize,
    pub focus: NewFocus,
    /// Selected input item (0 = symbolic_name, 1 = origin_path, 2 = target_path)
    pub selected: usize,
    pub editing: bool,
    /// Selected operation button (0 = 修改, 1 = 删除, 2 = 返回)
    pub button_idx: usize,
}

pub(crate) enum ManageAction {
    None,
    Edit,
    Commit,
    Delete,
    Back,
}

impl ManagePage {
    pub fn new(name: String, entry: SymbolicEntry, back_selected: usize) -> Self {
        Self {
            key: name.clone(),
            symbolic_name: name,
            origin_path: entry.origin_path,
            target_path: entry.target_path,
            add_datetime: entry.add_datetime,
            back_selected,
            focus: NewFocus::Buttons,
            selected: 0,
            editing: false,
            button_idx: 0,
        }
    }

    pub(crate) fn handle(&mut self, key: KeyEvent) -> ManageAction {
        // Esc: cancel editing / back to the buttons / back to the view page
        if key.code == KeyCode::Esc {
            if self.editing {
                self.editing = false;
                return ManageAction::None;
            }
            if self.focus == NewFocus::Input {
                self.focus = NewFocus::Buttons;
                return ManageAction::None;
            }
            return ManageAction::Back;
        }

        match self.focus {
            NewFocus::Buttons => {
                match key.code {
                    KeyCode::Left => self.button_idx = (self.button_idx + 2) % 3,
                    KeyCode::Right => self.button_idx = (self.button_idx + 1) % 3,
                    KeyCode::Tab => {
                        self.focus = NewFocus::Input;
                        self.selected = 0;
                        self.editing = false;
                    }
                    KeyCode::Enter => {
                        return match self.button_idx {
                            // 修改: switch to the input area
                            0 => {
                                self.focus = NewFocus::Input;
                                self.selected = 0;
                                self.editing = false;
                                ManageAction::Edit
                            }
                            // 删除: ask for a confirmation
                            1 => ManageAction::Delete,
                            // 返回: back to the view page
                            _ => ManageAction::Back,
                        };
                    }
                    _ => {}
                }
                ManageAction::None
            }
            NewFocus::Input => {
                if self.editing {
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
                            self.editing = false;
                            // Enter confirms the edit: save and stay in the
                            // input area, so the user can keep editing other
                            // fields; use Tab to get back to the buttons.
                            return ManageAction::Commit;
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Up => self.selected = self.selected.saturating_sub(1),
                        KeyCode::Down => self.selected = (self.selected + 1).min(2),
                        KeyCode::Enter => self.editing = true,
                        KeyCode::Tab => self.focus = NewFocus::Buttons,
                        _ => {}
                    }
                }
                ManageAction::None
            }
        }
    }
}

pub fn handle_manage(app: &mut App, key: KeyEvent) {
    let action = if let Page::Manage(p) = &mut app.page {
        p.handle(key)
    } else {
        return;
    };
    match action {
        ManageAction::None | ManageAction::Edit => {}
        ManageAction::Commit => {
            let (key, name, origin, target, dt) = if let Page::Manage(p) = &app.page {
                (
                    p.key.clone(),
                    p.symbolic_name.clone(),
                    p.origin_path.clone(),
                    p.target_path.clone(),
                    p.add_datetime.clone(),
                )
            } else {
                return;
            };
            // Expand a leading ~ / $HOME in the target path as well
            let target = expand_path(&target, &app.home);
            // 源路径可能被修改，确保新源路径对应的文件夹存在（含嵌套目录）
            if !origin.is_empty() {
                let _ = std::fs::create_dir_all(&origin);
            }
            // Remove the old key, then insert under the current name
            app.config.symbolic.remove(&key);
            app.config.symbolic.insert(
                name,
                SymbolicEntry {
                    origin_path: origin,
                    target_path: target,
                    add_datetime: dt,
                },
            );
            app.save_config();
            // The key now follows the new name for subsequent commits, and the
            // display stays in sync with the expanded target path
            if let Page::Manage(p) = &mut app.page {
                p.key = p.symbolic_name.clone();
                p.target_path = app.config.symbolic[p.symbolic_name.as_str()]
                    .target_path
                    .clone();
            }
        }
        ManageAction::Delete => {
            let name = if let Page::Manage(p) = &app.page {
                p.symbolic_name.clone()
            } else {
                return;
            };
            app.show_confirm(
                format!("确认删除软链接「{name}」？"),
                ConfirmAction::DeleteEntry(name),
            );
        }
        ManageAction::Back => {
            let idx = if let Page::Manage(p) = &app.page {
                p.back_selected
            } else {
                return;
            };
            let mut vp = ViewPage::from_config(&app.config);
            vp.selected = idx.min(vp.entries.len().saturating_sub(1));
            app.page = Page::View(vp);
        }
    }
}

pub fn draw_manage(frame: &mut Frame, p: &ManagePage) {
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
        frame.render_widget(
            Paragraph::new(value.as_str()).style(box_style),
            box_area_row,
        );
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

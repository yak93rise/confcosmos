use std::path::Path;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::{App, Page};
use crate::config::{Config, SymbolicEntry};
use crate::pages::NewFocus;
use crate::ui::{box_and_hints, draw_hints, HIGHLIGHT};

// ---------------------------------------------------------------------------
// generate page
// ---------------------------------------------------------------------------

pub enum GenStatus {
    Success,
    Exists,
    Failed(String),
}

pub struct GeneratePage {
    /// All saved entries (multi-select list)
    pub entries: Vec<(String, SymbolicEntry)>,
    /// Checkbox state per entry
    pub checks: Vec<bool>,
    /// List cursor
    pub cursor: usize,
    pub mode: GenMode,
    pub results: Vec<(String, GenStatus)>,
    /// Input(list) / Buttons focus
    pub focus: NewFocus,
    /// Selected operation button (0 = 确认生成, 1 = 返回)
    pub button_idx: usize,
    /// Transient message, e.g. "未选择任何软链接"
    pub notice: Option<String>,
}

pub enum GenMode {
    Select,
    Results,
}

impl GeneratePage {
    pub fn from_config(config: &Config) -> Self {
        let entries: Vec<(String, SymbolicEntry)> = config
            .symbolic
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let checks = vec![false; entries.len()];
        Self {
            entries,
            checks,
            cursor: 0,
            mode: GenMode::Select,
            results: Vec::new(),
            focus: NewFocus::Input,
            button_idx: 0,
            notice: None,
        }
    }
}

enum GenAction {
    None,
    Back,
    /// 生成完成，把成功创建的条目标记为 already_generate = true
    MarkGenerated,
}

pub fn handle_generate(app: &mut App, key: KeyEvent) {
    if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
        app.go_main();
        return;
    }
    let action = if let Page::Generate(p) = &mut app.page {
        match p.mode {
            GenMode::Results => {
                if key.code == KeyCode::Enter {
                    // back to the selection list
                    p.mode = GenMode::Select;
                }
                GenAction::None
            }
            GenMode::Select => {
                p.notice = None;
                if p.focus == NewFocus::Buttons {
                    match key.code {
                        KeyCode::Left | KeyCode::Right => {
                            p.button_idx = (p.button_idx + 1) % 2;
                            GenAction::None
                        }
                        KeyCode::Tab => {
                            p.focus = NewFocus::Input;
                            GenAction::None
                        }
                        KeyCode::Enter => match p.button_idx {
                            // 确认生成: generate the checked entries
                            0 => {
                                if p.checks.iter().any(|&c| c) {
                                    p.results = generate_selected(&p.checks, &p.entries);
                                    p.mode = GenMode::Results;
                                    GenAction::MarkGenerated
                                } else {
                                    p.notice = Some("未选择任何软链接".to_string());
                                    GenAction::None
                                }
                            }
                            // 返回
                            _ => GenAction::Back,
                        },
                        _ => GenAction::None,
                    }
                } else {
                    // list focus
                    match key.code {
                        KeyCode::Up => p.cursor = p.cursor.saturating_sub(1),
                        KeyCode::Down => {
                            p.cursor = (p.cursor + 1).min(p.entries.len().saturating_sub(1))
                        }
                        KeyCode::Char(' ') | KeyCode::Enter => {
                            if let Some(c) = p.checks.get_mut(p.cursor) {
                                *c = !*c;
                            }
                        }
                        KeyCode::Tab => p.focus = NewFocus::Buttons,
                        _ => {}
                    }
                    GenAction::None
                }
            }
        }
    } else {
        return;
    };
    match action {
        GenAction::None => {}
        GenAction::Back => app.go_main(),
        GenAction::MarkGenerated => mark_generated(app),
    }
}

/// 生成结束后，把结果中成功创建的条目标记为 already_generate = true
/// 并保存配置（结果保存在 GeneratePage.results 里，动作处理时页面借已释放）。
fn mark_generated(app: &mut App) {
    let generated: Vec<String> = match &app.page {
        Page::Generate(p) => p
            .results
            .iter()
            .filter(|(_, st)| matches!(st, GenStatus::Success))
            .map(|(name, _)| name.clone())
            .collect(),
        _ => return,
    };
    let mut changed = false;
    for name in generated {
        if let Some(entry) = app.config.symbolic.get_mut(&name) {
            if !entry.already_generate {
                entry.already_generate = true;
                changed = true;
            }
        }
    }
    if changed {
        app.save_config();
    }
}

pub fn draw_generate(frame: &mut Frame, p: &GeneratePage) {
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
                            GenStatus::Exists => {
                                (format!("{name} : 目标已存在，跳过"), Color::Yellow)
                            }
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

/// Create the symlinks (target_path -> origin_path) for every configured entry.
/// Generate the symlinks for the checked entries only.
fn generate_selected(
    checks: &[bool],
    entries: &[(String, SymbolicEntry)],
) -> Vec<(String, GenStatus)> {
    let mut results = Vec::new();
    for (i, (name, entry)) in entries.iter().enumerate() {
        if i < checks.len() && checks[i] {
            results.push((name.clone(), generate_entry(entry)));
        }
    }
    results
}

fn generate_entry(entry: &SymbolicEntry) -> GenStatus {
    let target = Path::new(&entry.target_path);
    if target.symlink_metadata().is_ok() {
        return GenStatus::Exists;
    }
    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
    match std::os::unix::fs::symlink(&entry.origin_path, target) {
        Ok(()) => GenStatus::Success,
        Err(e) => GenStatus::Failed(e.to_string()),
    }
}

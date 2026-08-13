use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent};

use crate::config::{expand_path, join_origin_path, Config, SymbolicEntry};

pub const MENU_ITEMS: [&str; 4] = ["新增软链接", "查看软链接", "生成软链接", "设置"];

// ---------------------------------------------------------------------------
// Pages
// ---------------------------------------------------------------------------

pub enum Page {
    Main,
    Setup(SetupPage),
    New(NewPage),
    View(ViewPage),
    Manage(ManagePage),
    Generate(GeneratePage),
    Settings(SettingsPage),
    Confirm(ConfirmPage),
}

/// First-run page: ask for the source path prefix.
pub struct SetupPage {
    pub input: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewFocus {
    Input,
    Buttons,
}

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

/// `symbolic_view_page` state.
pub struct ViewPage {
    pub entries: Vec<(String, SymbolicEntry)>,
    pub selected: usize,
}

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

pub struct SettingsPage {
    pub input: String,
    pub saved: bool,
}

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

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

pub struct App {
    pub quit: bool,
    pub menu_selected: usize,
    pub config: Config,
    pub config_path: PathBuf,
    pub home: String,
    pub page: Page,
    /// The page that was active before a confirm dialog was opened
    pub prev: Option<Page>,
}

impl App {
    pub fn new(config: Config, config_path: PathBuf, home: String, needs_setup: bool) -> Self {
        let page = if needs_setup {
            Page::Setup(SetupPage { input: String::new() })
        } else {
            Page::Main
        };
        Self {
            quit: false,
            menu_selected: 0,
            config,
            config_path,
            home,
            page,
            prev: None,
        }
    }

    pub fn go_main(&mut self) {
        self.page = Page::Main;
    }

    pub fn show_confirm(&mut self, message: String, action: ConfirmAction) {
        let prev = std::mem::replace(
            &mut self.page,
            Page::Confirm(ConfirmPage {
                message,
                action,
                selected: 0,
            }),
        );
        self.prev = Some(prev);
    }

    fn save_config(&self) {
        let _ = self.config.save(&self.config_path);
    }
}

// ---------------------------------------------------------------------------
// Event handling
// ---------------------------------------------------------------------------

pub fn handle(app: &mut App, key: KeyEvent) {
    match &app.page {
        Page::Main => handle_main(app, key),
        Page::Setup(_) => handle_setup(app, key),
        Page::New(_) => handle_new(app, key),
        Page::View(_) => handle_view(app, key),
        Page::Manage(_) => handle_manage(app, key),
        Page::Generate(_) => handle_generate(app, key),
        Page::Settings(_) => handle_settings(app, key),
        Page::Confirm(_) => handle_confirm(app, key),
    }
}

fn handle_main(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up => app.menu_selected = app.menu_selected.saturating_sub(1),
        KeyCode::Down => app.menu_selected = (app.menu_selected + 1).min(MENU_ITEMS.len() - 1),
        KeyCode::Enter => match app.menu_selected {
            0 => app.page = Page::New(NewPage::new()),
            1 => {
                let entries: Vec<(String, SymbolicEntry)> = app
                    .config
                    .symbolic
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                app.page = Page::View(ViewPage { entries, selected: 0 });
            }
            2 => app.page = Page::Generate(GeneratePage::from_config(&app.config)),
            _ => app.page = Page::Settings(SettingsPage {
                input: app.config.origin_path_prefix.clone(),
                saved: false,
            }),
        },
        KeyCode::Char('q') => app.quit = true,
        _ => {}
    }
}

fn handle_setup(app: &mut App, key: KeyEvent) {
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

pub(crate) enum NewResult {
    None,
    Abandon,
    ConfirmAbandon,
    Save,
}

impl NewPage {
    pub fn new() -> Self {        Self {
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
                                    self.notice = Some("内容不能为空，若放弃新增可按 Esc 键返回主页".to_string());
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
                                self.notice = Some("内容不能为空，若放弃新增可按 Esc 键返回主页".to_string());
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
                                self.notice = Some("内容不能为空，若放弃新增可按 Esc 键返回主页".to_string());
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

fn handle_new(app: &mut App, key: KeyEvent) {
    let action = if let Page::New(p) = &mut app.page {
        p.handle(key)
    } else {
        return;
    };
    match action {
        NewResult::None => {}
        NewResult::Abandon => app.go_main(),
        NewResult::ConfirmAbandon => {
            app.show_confirm("确认放弃新增并返回主页面？".to_string(), ConfirmAction::AbandonNew);
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

    pub fn handle(&mut self, key: KeyEvent) -> ManageAction {
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

fn handle_manage(app: &mut App, key: KeyEvent) {
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
            let entries: Vec<(String, SymbolicEntry)> = app
                .config
                .symbolic
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let sel = idx.min(entries.len().saturating_sub(1));
            app.page = Page::View(ViewPage { entries, selected: sel });
        }
    }
}

fn handle_view(app: &mut App, key: KeyEvent) {
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

enum GenAction {
    None,
    Back,
}

fn handle_generate(app: &mut App, key: KeyEvent) {
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
                                    p.results =
                                        generate_selected(&p.checks, &p.entries);
                                    p.mode = GenMode::Results;
                                } else {
                                    p.notice = Some("未选择任何软链接".to_string());
                                }
                                GenAction::None
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
    }
}

fn handle_settings(app: &mut App, key: KeyEvent) {
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

enum ConfirmChoice {
    Yes,
    No,
    None,
}

fn handle_confirm(app: &mut App, key: KeyEvent) {
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
                    let entries: Vec<(String, SymbolicEntry)> = app
                        .config
                        .symbolic
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    app.page = Page::View(ViewPage { entries, selected: 0 });
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

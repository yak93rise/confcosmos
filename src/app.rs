use std::path::PathBuf;

use crossterm::event::KeyEvent;

use crate::config::Config;
use crate::pages;
use crate::pages::confirm_page::{ConfirmAction, ConfirmPage};
use crate::pages::generate_page::GeneratePage;
use crate::pages::manage_page::ManagePage;
use crate::pages::new_page::NewPage;
use crate::pages::settings_page::SettingsPage;
use crate::pages::setup_page::SetupPage;
use crate::pages::view_page::ViewPage;

pub const MENU_ITEMS: [&str; 4] = ["查看软链接", "新增软链接", "生成软链接", "设置"];

// ---------------------------------------------------------------------------
// Pages
// ---------------------------------------------------------------------------

/// 当前激活的页面。每个页面的状态、事件处理和渲染都在
/// `src/pages/<name>_page.rs` 中；本枚举是 `app::handle` 和 `ui::draw`
/// 统一调度的枢纽。
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
            Page::Setup(SetupPage {
                input: String::new(),
            })
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

    pub(crate) fn save_config(&self) {
        let _ = self.config.save(&self.config_path);
    }
}

// ---------------------------------------------------------------------------
// Event handling (dispatch to the active page)
// ---------------------------------------------------------------------------

pub fn handle(app: &mut App, key: KeyEvent) {
    match &app.page {
        Page::Main => pages::main_page::handle_main(app, key),
        Page::Setup(_) => pages::setup_page::handle_setup(app, key),
        Page::New(_) => pages::new_page::handle_new(app, key),
        Page::View(_) => pages::view_page::handle_view(app, key),
        Page::Manage(_) => pages::manage_page::handle_manage(app, key),
        Page::Generate(_) => pages::generate_page::handle_generate(app, key),
        Page::Settings(_) => pages::settings_page::handle_settings(app, key),
        Page::Confirm(_) => pages::confirm_page::handle_confirm(app, key),
    }
}

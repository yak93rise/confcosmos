//! 每个页面一个模块。页面模块自包含「状态结构体、事件处理 (handle_<name>)、
//! 渲染 (draw_<name>)」；`app::handle` 和 `ui::draw` 通过 `Page` 枚举统一分发到各页面。
//!
//! 页面列表：`confirm_page` 确认对话框、`generate_page` 生成软链接、
//! `main_page` 主菜单、`manage_page` 管理单个软链接、`new_page` 新增软链接、
//! `settings_page` 设置、`setup_page` 首次运行设置、`view_page` 查看软链接。

pub mod confirm_page;
pub mod generate_page;
pub mod main_page;
pub mod manage_page;
pub mod new_page;
pub mod settings_page;
pub mod setup_page;
pub mod view_page;

/// 输入区域 / 操作按钮行 的共享焦点状态
/// （被 new 页面和 manage 页面共用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewFocus {
    Input,
    Buttons,
}

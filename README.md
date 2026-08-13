# ConfCosmos

一个使用

- [pi coding agent](https://github.com/earendil-works/pi)
- Rust 编程语言
- [ratatui](https://github.com/ratatui/ratatui)

开发的 TUI 工具，用于快速创建软链接，意图在同一个位置管理散乱的配置文件，灵感源自 nix 和 home-manager。

仅支持 Linux。

## 构建与运行

```bash
cargo build --release
./target/release/confcosmos
```

## 配置

配置文件位于 `~/.config/confcosmos/confcosmos.toml`，首次启动时自动创建：

- 首次运行会询问「源路径前缀路径」（`origin_path_prefix`），之后新增软链接时输入的源路径会自动拼接此前缀。
- 输入以 `~` 或 `$HOME` 开头时自动展开为家目录路径。

```toml
origin_path_prefix = "/home/user/dotfiles"

[symbolic.vimrc]
origin_path = "/home/user/dotfiles/.vimrc"
target_path = "/home/user/.vimrc"
add_datetime = "2026-08-13 12:15:56 390"
```

## 操作

| 页面 | 按键 |
| --- | --- |
| 主菜单 | ↑/↓ 选择，Enter 确认，q 退出 |
| 新增软链接 | ↑/↓ 选择项目，←/→ 选择操作按钮，Enter 确认，Tab 切换输入区域与操作按钮，Esc 放弃并返回 |
| 查看软链接 | ↑/↓ 选择项目，Esc/q 返回 |
| 生成软链接 | Enter 重新生成，Esc/q 返回 |
| 设置 | Enter 保存，Esc/q 返回 |
| 确认弹窗 | y/Enter 确认，n/Esc 取消 |

## 功能

- **新增软链接**：依次输入软链接名称、源路径、目标路径，确认后写入配置。
- **查看软链接**：列出所有配置项，聚焦某一项时显示其源路径与目标路径。
- **生成软链接**：根据配置批量创建软链接（目标路径 -> 源路径），已存在的目标自动跳过。
- **设置**：修改源路径前缀路径。

## 测试

```bash
cargo test
```

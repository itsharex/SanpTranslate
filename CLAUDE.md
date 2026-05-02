# CLAUDE.md

本文件为 Claude Code（claude.ai/code）在此仓库中工作时提供指引。

## 项目概述

SnapTranslate 是一款基于 Tauri 2.x 的桌面截屏翻译工具。它能截取屏幕区域、执行 OCR（Tesseract）或多模态 AI 翻译，并将译文覆盖在原截图上方，以贴图形式固定在桌面上。

**当前状态：** 早期开发阶段 — 配置管理、系统托盘和窗口管理已完成。OCR、翻译、截图、剪贴板、历史记录和快捷键模块均为桩代码，等待实现。

## 开发命令

```bash
# 开发模式（Vite HMR + Tauri）
npm run tauri dev

# 构建生产版本
npm run tauri build

# 仅前端（Vite 开发服务器）
npm run dev

# 前端类型检查
npx vue-tsc --noEmit

# Vite 预览
npm run preview
```

## 架构

### 运行时流程

1. 用户按下全局快捷键（`Ctrl+Shift+X`）→ 截图模块截取屏幕
2. 截图以透明窗口形式固定在原位置（贴图窗口）
3. 用户点击"AI 翻译"→ OCR 提取文本及坐标，或多模态 API 直接翻译图像
4. 译文以标签形式覆盖在原文本位置上
5. 翻译记录保存到本地 SQLite 数据库

### Rust 后端（`src-tauri/src/`）

| 模块 | 文件 | 状态 |
|--------|------|--------|
| `capture` | `capture/mod.rs` | 桩代码 — 将使用 `xcap` 截取屏幕区域 |
| `ocr` | `ocr/mod.rs` | 桩代码 — 将使用 Tesseract（leptess）提取文本及词级坐标 |
| `translate` | `translate/mod.rs` | 桩代码 — 将通过 reqwest 调用 AI API，支持 OCR 模式（纯文本）和多模态模式（图像到文本） |
| `clipboard` | `clipboard/mod.rs` | 桩代码 — 将使用 `tauri-plugin-clipboard-manager` |
| `hotkey` | `hotkey/mod.rs` | 桩代码 — 将使用 `tauri-plugin-global-shortcut` |
| `history` | `history/mod.rs` | 桩代码 — 将使用 `rusqlite` 进行 SQLite 存储 |
| `config` | `config/manager.rs` | **已完成** — 基于 TOML 的配置（API URL、模型名称、语言、快捷键），从 `app_config_dir/config.toml` 加载，通过临时文件+重命名实现原子写入 |
| `config` | `config/mod.rs` | 重新导出 `AppConfig`、`ConfigManager`、`ShortcutConfig`、`TranslateMode` |
| `window` | `window/mod.rs` | **已完成** — 通过 `WebviewWindowBuilder` 创建设置和历史记录窗口；覆盖层和贴图窗口函数为桩代码 |
| `tray` | `tray/mod.rs` | **已完成** — 系统托盘菜单：框选截图、从剪贴板贴图、翻译最近贴图、历史记录、设置、退出 |
| `commands` | `commands.rs` | **已完成** — 两个 Tauri 命令：`get_config` 和 `save_config`，均委托给 `ConfigManager` |
| `error` | `error.rs` | **已完成** — 统一的 `AppError` 枚举，包含 `io`、`toml`、`reqwest`、`rusqlite`、`tauri` 的 `From` 实现 |
| `lib.rs` | — | 应用入口点：注册 opener 插件、配置命令、在 `setup()` 中创建系统托盘 |
| `main.rs` | — | `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`，调用 `snap_translate_lib::run()` |

**关键后端模式：** 所有模块在其目录下均为扁平化的 `mod.rs` 文件。目前只有 config 采用了拆分模块结构（config/mod.rs + config/manager.rs）。

### 前端（`src/`）

| 层级 | 文件 | 用途 |
|-------|------|---------|
| 入口 | `main.ts` | 创建 Vue 应用，注册 router/Pinia/i18n |
| 路由 | `router/index.ts` | 四个路由：`/overlay`（截图蒙版）、`/pin`（贴图）、`/settings`、`/history` |
| Pinia | `stores/configStore.ts` | 配置状态（通过 `invoke` 与 Rust 后端进行加载/保存） |
| Pinia | `stores/pinStore.ts` | 贴图状态管理（`TranslatedBlock`、`PinState` 及贴图实例的 Map） |
| Pinia | `stores/historyStore.ts` | 历史记录状态（桩代码） |
| 国际化 | `i18n/index.ts` | `vue-i18n` 配置，自动检测 zh-CN 或 en-US |
| 国际化 | `i18n/locales/` | 两个语言文件当前均为空对象 |
| 工具函数 | `utils/tauri.ts` | Tauri 命令的 TypeScript 绑定（`getConfig`、`saveConfig`） |
| 样式 | `styles/variables.css` | CSS 自定义属性（深色透明主题） |
| 样式 | `styles/global.css` | 全局重置及基础样式 |
| 视图 | `views/OverlayView.vue` | 截图框选蒙版（桩代码） |
| 视图 | `views/PinView.vue` | 贴图窗口（桩代码 — 显示"贴图窗口"） |
| 视图 | `views/SettingsView.vue` | 设置页面（桩代码） |
| 视图 | `views/HistoryView.vue` | 历史记录页面（桩代码） |

**关键前端模式：** 使用 `@/` 路径别名（在 vite.config.ts 和 tsconfig.json 中均已配置）。通过 `@tauri-apps/api/core` 的 `invoke()` 进行 Tauri IPC 通信。所有视图均为懒加载。

### AppConfig 结构（Rust ↔ TypeScript）

```rust
pub struct AppConfig {
    pub api_base_url: String,
    pub text_model: String,           // 例如 "gpt-4o"
    pub vision_model: Option<String>,  // 例如 "gpt-4o-vision"
    pub target_language: String,      // 默认 "zh-CN"
    pub default_mode: TranslateMode,  // Ocr | Multimodal
    pub shortcuts: ShortcutConfig,    // capture: "Ctrl+Shift+X", pin_clipboard: "Ctrl+Shift+V"
}
```

### 关键设计决策

- **配置存储：** TOML 文件存储在磁盘上，API 密钥通过操作系统凭据管理器（`keyring` crate）保存 — 不写入配置文件
- **翻译模式：** "Ocr" 模式 → 使用 OCR 提取的文本进行纯文本 API 调用；"Multimodal" 模式 → 直接将图像发送到 API 进行图像到文本翻译
- **贴图窗口：** 每张贴图截图都是一个独立的透明 Tauri Webview 窗口，定位在原始截取坐标处
- **错误处理：** 统一的 `AppError` 枚举，`Display` 输出中文错误信息，包含所有主要错误类型的 `From` 实现
- **窗口管理：** 设置和历史记录窗口为单例（如果已打开则复用现有窗口）
- **UI 框架：** Naive UI 已声明为依赖项，但尚未在任何组件中使用

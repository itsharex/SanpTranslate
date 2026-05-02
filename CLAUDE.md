# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

- 在回答过程中请使用中文
- 在代码中添加中文注释

***

## Reasoning Effort: 
Absolute maximum with no shortcuts permitted.
You MUST be very thorough in your thinking and comprehensively decompose the problem to resolve the root cause, rigorously stress-testing your logic against all potential paths, edge cases, and adversarial scenarios.
Explicitly write out your entire deliberation process, documenting every intermediate step, considered alternative, and rejected hypothesis to ensure absolutely no assumption is left unchecked.

## 项目概述

SnapTranslate 是一款基于 Tauri 2.x 的桌面截屏翻译工具。它能截取屏幕区域、执行 OCR（Tesseract）或多模态 AI 翻译，并将译文覆盖在原截图上方，以贴图形式固定在桌面上。

**当前状态：** S2 阶段 — 截图、剪贴板、快捷键、贴图窗口、框选蒙版、托盘菜单均已实现。OCR、翻译、历史记录模块仍为桩代码。

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

| 模块          | 文件                  | 状态                                                                                                                                                                                                  |
| ----------- | ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `capture`   | `capture/mod.rs`    | **已完成** — `CaptureService` 封装 xcap，支持全屏截图和区域截图，返回 Base64 PNG；包含 `MonitorInfo`、`CaptureRegion`、`CapturedImage` 数据结构                                                                                  |
| `ocr`       | `ocr/mod.rs`        | 桩代码 — 将使用 Tesseract（leptess）提取文本及词级坐标                                                                                                                                                               |
| `translate` | `translate/mod.rs`  | 桩代码 — 将通过 reqwest 调用 AI API，支持 OCR 模式（纯文本）和多模态模式（图像到文本）                                                                                                                                             |
| `clipboard` | `clipboard/mod.rs`  | **已完成** — `read_clipboard_image`/`write_clipboard_image`/`write_clipboard_text`，通过 Base64 ↔ PNG ↔ tauri Image 转换读写图片                                                                                |
| `hotkey`    | `hotkey/mod.rs`     | **已完成** — `register_hotkeys` 注册全局快捷键（从配置动态解析），支持 Ctrl/Shift/Alt/Super 修饰键 + A-Z/0-9/F1-F12，回调中串联截图或剪贴板操作                                                                                            |
| `history`   | `history/mod.rs`    | 桩代码 — 将使用 `rusqlite` 进行 SQLite 存储                                                                                                                                                                   |
| `config`    | `config/manager.rs` | **已完成** — 基于 TOML 的配置（API URL、模型名称、语言、快捷键），从 `app_config_dir/config.toml` 加载，通过临时文件+重命名实现原子写入                                                                                                       |
| `config`    | `config/mod.rs`     | 重新导出 `AppConfig`、`ConfigManager`、`ShortcutConfig`、`TranslateMode`                                                                                                                                   |
| `window`    | `window/mod.rs`     | **已完成** — `create_settings_window`/`create_history_window`（单例模式）、`create_overlay_window`（全屏蒙版，通过事件发射截图数据）、`create_pin_window`（UUID 标签，窗口尺寸预留控制栏高度）、`close_pin_window`                               |
| `tray`      | `tray/mod.rs`       | **已完成** — 系统托盘菜单：框选截图、从剪贴板贴图、翻译最近贴图、历史记录、设置、退出。capture 和 pin\_clipboard 菜单项已接入实际逻辑                                                                                                                  |
| `commands`  | `commands.rs`       | **已完成** — 九个 Tauri 命令：`get_config`、`save_config`、`capture_fullscreen`、`capture_region`、`write_clipboard_image`、`read_clipboard_image`、`write_clipboard_text`、`create_pin_window`、`close_pin_window` |
| `error`     | `error.rs`          | **已完成** — 统一的 `AppError` 枚举，`Display` 输出中文错误信息，包含 `io`、`toml`、`reqwest`、`rusqlite`、`tauri` 的 `From` 实现                                                                                              |
| `lib.rs`    | —                   | 应用入口点：注册 opener 插件和 clipboard\_manager 插件、注册所有命令、在 `setup()` 中创建系统托盘并注册全局快捷键                                                                                                                        |
| `main.rs`   | —                   | `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`，调用 `snap_translate_lib::run()`                                                                                                 |

**关键后端模式：** 所有模块在其目录下均为扁平化的 `mod.rs` 文件。目前只有 config 采用了拆分模块结构（config/mod.rs + config/manager.rs）。

### 前端（`src/`）

| 层级    | 文件                          | 用途                                                                            |
| ----- | --------------------------- | ----------------------------------------------------------------------------- |
| 入口    | `main.ts`                   | 创建 Vue 应用，注册 router/Pinia/i18n                                                |
| 路由    | `router/index.ts`           | 四个路由：`/overlay`（截图蒙版）、`/pin`（贴图）、`/settings`、`/history`                       |
| 组件    | `components/ControlBar.vue` | 贴图控制栏组件：根据翻译状态（idle/translating/done/error）显示 AI 翻译按钮、翻译中状态、复制全部/切换原文/译文面板按钮组 |
| Pinia | `stores/configStore.ts`     | 配置状态（通过 `invoke` 与 Rust 后端进行加载/保存）                                            |
| Pinia | `stores/pinStore.ts`        | 贴图状态管理（`TranslatedBlock`、`PinState` 及贴图实例的 Map）                               |
| Pinia | `stores/historyStore.ts`    | 历史记录状态（桩代码）                                                                   |
| 国际化   | `i18n/index.ts`             | `vue-i18n` 配置，自动检测 zh-CN 或 en-US                                              |
| 国际化   | `i18n/locales/`             | 两个语言文件当前均为空对象                                                                 |
| 工具函数  | `utils/tauri.ts`            | Tauri 命令的 TypeScript 绑定（所有九个命令均已覆盖）                                           |
| 工具函数  | `utils/logger.ts`           | 日志工具，封装 `@tauri-apps/plugin-log`，提供带时间戳和标签的 debug/info/warn/error 结构化日志输出 |
| 样式    | `styles/variables.css`      | CSS 自定义属性（深色透明主题）                                                             |
| 样式    | `styles/global.css`         | 全局重置及基础样式                                                                     |
| 视图    | `views/OverlayView.vue`     | **已完成** — Canvas 全屏截图蒙版，支持鼠标框选（白虚线框+暗色蒙版）、尺寸提示、Esc 关闭；选后调用后端裁剪+写入剪贴板+创建贴图窗口   |
| 视图    | `views/PinView.vue`         | **已完成** — 贴图窗口：显示截图、控制栏组件（翻译/复制/切换）、原生窗口拖拽（排除按钮区域）、双击图片区域关闭                   |
| 视图    | `views/SettingsView.vue`    | 设置页面（桩代码）                                                                     |
| 视图    | `views/HistoryView.vue`     | 历史记录页面（桩代码）                                                                   |

**关键前端模式：** 使用 `@/` 路径别名（在 vite.config.ts 和 tsconfig.json 中均已配置）。通过 `@tauri-apps/api/core` 的 `invoke()` 进行 Tauri IPC 通信。Tauri 窗口间数据传递使用 Event 机制（`emit`/`listen`）。所有视图均为懒加载。

### 前端通信模式

- **蒙版窗口流程：** `lib.rs` setup → 快捷键回调 → `capture::capture_fullscreen()` → `window::create_overlay_window()` → 后端 `emit("overlay-image", base64)` → OverlayView 监听 `overlay-image` 事件 → 绘制截图 → 用户框选 → `capture_region()` → `write_clipboard_image()` → `create_pin_window()`
- **贴图窗口流程：** `create_pin_window()` 创建 WebviewWindow → 后端 `emit_to(label, "pin-image", base64)` → PinView 监听 `pin-image` 事件 → 显示图片 + ControlBar

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
- **贴图窗口：** 每张贴图截图都是一个独立的透明 Tauri Webview 窗口，定位在原始截取坐标处，窗口尺寸 = 图片高度 + 36px 控制栏高度
- **错误处理：** 统一的 `AppError` 枚举，`Display` 输出中文错误信息，包含所有主要错误类型的 `From` 实现
- **窗口管理：** 设置和历史记录窗口为单例（如果已打开则复用现有窗口）；蒙版窗口为单例（打开前先关闭已有实例）；贴图窗口每次创建新实例（UUID 标签）
- **UI 框架：** Naive UI 已声明为依赖项，但尚未在任何组件中使用


import { invoke } from '@tauri-apps/api/core'

/** 快捷键配置 */
export interface ShortcutConfig {
  /** 截图翻译快捷键 */
  capture: string
  /** 固定到剪贴板快捷键 */
  pin_clipboard: string
}

/** 应用配置，与 Rust 后端 AppConfig 结构保持一致 */
export interface AppConfig {
  /** API 基础地址 */
  api_base_url: string
  /** 文本模型名称 */
  text_model: string
  /** 视觉模型名称（可选，多模态模式需要） */
  vision_model: string | null
  /** 目标语言 */
  target_language: string
  /** 默认翻译模式 */
  default_mode: 'Ocr' | 'Multimodal'
  /** 快捷键配置 */
  shortcuts: ShortcutConfig
}

/** 获取应用配置 */
export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config')
}

/** 保存应用配置 */
export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke('save_config', { config })
}

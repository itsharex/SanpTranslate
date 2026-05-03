import { invoke } from '@tauri-apps/api/core'

export interface AppConfig {
  shortcuts: ShortcutConfig
  translation: TranslationConfig
  ocr: OcrConfig
  general: GeneralConfig
}

export interface ShortcutConfig {
  capture: string
  pin_clipboard: string
  translate_recent: string
}

export interface TranslationConfig {
  default_mode: string
  auto_translate: boolean
  source_lang: string
  target_lang: string
}

export interface OcrConfig {
  engine: string
  language: string
}

export interface GeneralConfig {
  theme: string
  language: string
  startup: boolean
  show_tray: boolean
}

/** 区域裁剪结果，包含图像数据和窗口位置信息 */
export interface CropResult {
  /** Base64 编码的 PNG 图像数据 */
  base64_data: string
  /** 贴图窗口 X 位置（逻辑像素） */
  x: number
  /** 贴图窗口 Y 位置（逻辑像素） */
  y: number
  /** 贴图窗口宽度（逻辑像素，含内边距） */
  width: number
  /** 贴图窗口高度（逻辑像素，含内边距和控制栏） */
  height: number
  /** 裁剪区域的物理像素宽度 */
  crop_width: number
  /** 裁剪区域的物理像素高度 */
  crop_height: number
}

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config')
}

export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke('save_config', { config })
}

export async function writeClipboardImage(imageData: string): Promise<void> {
  return invoke('write_clipboard_image', { imageData })
}

export async function readClipboardImage(): Promise<string | null> {
  return invoke<string | null>('read_clipboard_image')
}

export async function writeClipboardText(text: string): Promise<void> {
  return invoke('write_clipboard_text', { text })
}

export async function closePinWindow(windowId: string): Promise<void> {
  return invoke('close_pin_window', { windowId })
}

export async function getPinImage(windowId: string): Promise<string | null> {
  return invoke<string | null>('get_pin_image', { windowId })
}

// 从缓存的全屏截图中裁剪指定区域，返回裁剪结果（图像数据 + 位置信息）
export async function captureRegionFromCache(
  x: number,
  y: number,
  width: number,
  height: number
): Promise<CropResult> {
  return invoke<CropResult>('capture_region_from_cache', { x, y, width, height })
}

// 存储贴图图像数据到后端 PinImageStore，供 PinView 获取
export async function storePinImage(label: string, imageData: string): Promise<void> {
  return invoke('store_pin_image', { label, imageData })
}

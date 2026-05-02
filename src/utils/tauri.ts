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

export interface PinWindowInfo {
  label: string
  x: number
  y: number
  width: number
  height: number
}

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config')
}

export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke('save_config', { config })
}

export async function captureFullscreen(monitorId?: string): Promise<string> {
  return invoke<string>('capture_fullscreen', { monitorId })
}

export async function captureRegion(
  x: number,
  y: number,
  width: number,
  height: number,
  monitorId?: string
): Promise<string> {
  return invoke<string>('capture_region', { x, y, width, height, monitorId })
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

export async function createPinWindow(
  imageData: string,
  x: number,
  y: number,
  w: number,
  h: number
): Promise<PinWindowInfo> {
  return invoke<PinWindowInfo>('create_pin_window', { imageData, x, y, w, h })
}

export async function closePinWindow(windowId: string): Promise<void> {
  return invoke('close_pin_window', { windowId })
}

export async function getPinImage(windowId: string): Promise<string | null> {
  return invoke<string | null>('get_pin_image', { windowId })
}

import { debug, info, warn, error } from '@tauri-apps/plugin-log'

function formatMsg(tag: string, msg: string, data?: unknown): string {
  const ts = new Date().toISOString().slice(11, 23)
  const dataStr = data !== undefined ? ` | ${JSON.stringify(data)}` : ''
  return `[${ts}][${tag}] ${msg}${dataStr}`
}

export const logger = {
  debug(tag: string, msg: string, data?: unknown) {
    const formatted = formatMsg(tag, msg, data)
    console.log(formatted)
    debug(formatted).catch(() => {})
  },
  info(tag: string, msg: string, data?: unknown) {
    const formatted = formatMsg(tag, msg, data)
    console.info(formatted)
    info(formatted).catch(() => {})
  },
  warn(tag: string, msg: string, data?: unknown) {
    const formatted = formatMsg(tag, msg, data)
    console.warn(formatted)
    warn(formatted).catch(() => {})
  },
  error(tag: string, msg: string, data?: unknown) {
    const formatted = formatMsg(tag, msg, data)
    console.error(formatted)
    error(formatted).catch(() => {})
  },
}

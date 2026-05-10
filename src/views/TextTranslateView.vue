<template>
  <div class="text-translate-container">
    <!-- 可拖拽的标题栏 -->
    <div class="title-bar" data-tauri-drag-region @dblclick="onClose">
      <span class="title-text">{{ t('textTranslate.title') }}</span>
      <button class="close-btn" @click="onClose" :title="t('common.close')">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <!-- 输入区域 -->
    <div class="input-area">
      <div class="input-wrapper">
        <textarea
          ref="inputRef"
          v-model="inputText"
          class="text-input"
          :placeholder="t('textTranslate.inputPlaceholder')"
          @keydown.ctrl.enter="onTranslate"
          @input="onInputChange"
        ></textarea>
        <div class="shortcut-hint">{{ t('textTranslate.shortcutHint') }}</div>
      </div>
      <button
        class="translate-btn"
        :class="{ 'translate-btn-translating': translateStatus === 'translating' }"
        :disabled="translateStatus === 'translating' || !inputText.trim()"
        @click="onTranslateClick"
      >
        <span v-if="translateStatus === 'translating'">{{ t('textTranslate.translating') }}</span>
        <span v-else-if="translateStatus === 'done' || translateStatus === 'error'">{{ t('textTranslate.retranslate') }}</span>
        <span v-else>{{ t('textTranslate.translate') }}</span>
      </button>
    </div>

    <!-- 译文面板 -->
    <div v-if="hasTranslation" class="translation-panel">
      <!-- 面板头部 -->
      <div class="panel-header" @dblclick="onClose">
        <span v-if="fromCache" class="cache-hint">{{ t('controlBar.cacheHit') }}</span>
        <button class="copy-btn" :class="{ 'copy-btn-copied': copyFeedback }" @click="onCopyTranslation" :title="t('textTranslate.copyTranslation')">
          <svg v-if="!copyFeedback" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
          </svg>
          <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
        </button>
      </div>
      <!-- 译文内容 -->
      <div class="translation-content" ref="translationContentRef">
        {{ translatedText }}
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="errorMessage" class="error-message">
      {{ errorMessage }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  getConfig,
  translateText,
  writeClipboardText,
} from '@/utils/tauri'
import { logger } from '@/utils/logger'

const TAG = 'TextTranslateView'
const { t } = useI18n()

type TranslateStatus = 'idle' | 'translating' | 'done' | 'error'

const inputText = ref('')
const translateStatus = ref<TranslateStatus>('idle')
const translatedText = ref('')
const hasTranslation = ref(false)
const fromCache = ref(false)
const errorMessage = ref('')
const inputRef = ref<HTMLTextAreaElement | null>(null)
const translationContentRef = ref<HTMLElement | null>(null)
const copyFeedback = ref(false)

/** 翻译核心逻辑 */
async function doTranslate(forceRetranslate: boolean) {
  if (!inputText.value.trim()) return

  translateStatus.value = 'translating'
  errorMessage.value = ''

  try {
    const config = await getConfig()
    logger.info(TAG, `开始文本翻译，目标语言=${config.target_language}，强制重新翻译=${forceRetranslate}`)

    const result = await translateText(inputText.value.trim(), config.target_language, forceRetranslate)

    if (!result.translated_text) {
      logger.info(TAG, '翻译结果为空')
      translateStatus.value = 'idle'
      return
    }

    translatedText.value = result.translated_text
    hasTranslation.value = true
    translateStatus.value = 'done'
    fromCache.value = result.from_cache

    logger.info(TAG, `文本翻译完成，from_cache=${result.from_cache}`)
  } catch (err) {
    errorMessage.value = String(err)
    translateStatus.value = 'error'
    logger.error(TAG, `文本翻译失败: ${err}`, err)
  }
}

/** 翻译/重新翻译按钮点击 */
function onTranslateClick() {
  if (translateStatus.value === 'done' || translateStatus.value === 'error') {
    doTranslate(true)
  } else {
    doTranslate(false)
  }
}

/** Ctrl+Enter 快捷翻译 */
function onTranslate() {
  if (translateStatus.value === 'translating') return
  if (translateStatus.value === 'done' || translateStatus.value === 'error') {
    doTranslate(true)
  } else {
    doTranslate(false)
  }
}

/** 输入变化时重置状态 */
function onInputChange() {
  if (translateStatus.value === 'done' || translateStatus.value === 'error') {
    translateStatus.value = 'idle'
    hasTranslation.value = false
    translatedText.value = ''
    errorMessage.value = ''
    fromCache.value = false
  }
}

/** 复制译文到剪贴板 */
async function onCopyTranslation() {
  if (!translatedText.value) return
  try {
    await writeClipboardText(translatedText.value)
    logger.info(TAG, '译文已复制到剪贴板')
    // 显示复制成功反馈
    copyFeedback.value = true
    setTimeout(() => { copyFeedback.value = false }, 1500)
  } catch (err) {
    logger.error(TAG, `复制译文失败: ${err}`, err)
  }
}

/** 关闭窗口 */
async function onClose() {
  try {
    await getCurrentWindow().destroy()
  } catch (err) {
    logger.error(TAG, `关闭窗口失败: ${err}`, err)
  }
}

/** Esc 键关闭窗口的处理函数 */
async function handleEscKey(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    try {
      await getCurrentWindow().destroy()
    } catch (err) {
      logger.error(TAG, `Esc关闭失败: ${err}`, err)
    }
  }
}

onMounted(async () => {
  logger.info(TAG, 'TextTranslateView onMounted')
  // 自动聚焦输入框
  await nextTick()
  if (inputRef.value) {
    inputRef.value.focus()
  }

  // Esc 键关闭窗口
  document.addEventListener('keydown', handleEscKey)
})

onUnmounted(() => {
  // 清理 Esc 键监听器
  document.removeEventListener('keydown', handleEscKey)
})
</script>

<style scoped>
.text-translate-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  background: rgba(20, 20, 24, 0.95);
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  overflow: hidden;
}

.title-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  background: rgba(255, 255, 255, 0.04);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
  user-select: none;
}

.title-text {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  font-weight: 500;
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
  transition: all 0.15s;
}

.close-btn:hover {
  background: rgba(255, 80, 80, 0.3);
  color: #ff6b6b;
}

.input-area {
  display: flex;
  flex-direction: row;
  gap: 0;
  padding: 12px;
  flex-shrink: 0;
}

.input-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.text-input {
  flex: 1;
  min-height: 80px;
  max-height: 120px;
  padding: 10px 12px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  color: #f0f0f0;
  font-size: 14px;
  line-height: 1.6;
  resize: none;
  outline: none;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  user-select: text;
}

.shortcut-hint {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
  text-align: right;
  user-select: none;
}

.text-input:focus {
  border-color: rgba(255, 255, 255, 0.25);
  background: rgba(255, 255, 255, 0.08);
}

.text-input::placeholder {
  color: rgba(255, 255, 255, 0.3);
}

.text-input::-webkit-scrollbar {
  width: 4px;
}

.text-input::-webkit-scrollbar-track {
  background: transparent;
}

.text-input::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 2px;
}

.translate-btn {
  flex-shrink: 0;
  margin-left: 10px;
  padding: 0 20px;
  border: none;
  border-radius: 6px;
  background: #3a7bd5;
  color: #ffffff;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s;
  align-self: flex-end;
  height: 36px;
}

.translate-btn:hover:not(:disabled) {
  background: #4a8be5;
}

.translate-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.translate-btn-translating {
  background: #555;
}

.translation-panel {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 6px 12px;
  gap: 8px;
  flex-shrink: 0;
  user-select: none;
}

.cache-hint {
  font-size: 12px;
  color: #4caf50;
}

.copy-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
  transition: all 0.15s;
}

.copy-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  color: #e0e0e0;
}

.copy-btn-copied {
  background: rgba(76, 175, 80, 0.2);
  color: #4caf50;
}

.translation-content {
  flex: 1;
  padding: 8px 16px 16px 16px;
  font-size: 14px;
  line-height: 1.8;
  color: #f0f0f0;
  white-space: pre-wrap;
  word-break: break-word;
  overflow-y: auto;
  user-select: text;
}

.translation-content::-webkit-scrollbar {
  width: 4px;
}

.translation-content::-webkit-scrollbar-track {
  background: transparent;
}

.translation-content::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 2px;
}

.error-message {
  padding: 8px 16px;
  font-size: 12px;
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
  border-top: 1px solid rgba(239, 68, 68, 0.2);
  flex-shrink: 0;
}
</style>

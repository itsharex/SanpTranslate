<template>
  <div class="control-bar">
    <!-- idle 或 error 状态：显示 AI 翻译主按钮 -->
    <button
      v-if="translateStatus === 'idle' || translateStatus === 'error'"
      class="btn btn-primary"
      @click="$emit('translate')"
    >
      {{ translateStatus === 'error' ? '重新翻译' : 'AI翻译' }}
    </button>

    <!-- translating 状态：显示禁用的翻译中按钮 -->
    <button
      v-else-if="translateStatus === 'translating'"
      class="btn btn-primary"
      disabled
    >
      翻译中...
    </button>

    <!-- done 状态：显示操作按钮组 -->
    <template v-else-if="translateStatus === 'done'">
      <!-- 复制全部 -->
      <button class="btn" @click="$emit('copyAll')">复制全部</button>

      <!-- 原文/译文切换 -->
      <button class="btn" @click="$emit('toggleOriginal')">
        {{ showOriginal ? '显示译文' : '显示原文' }}
      </button>

      <!-- 多模态模式下显示译文面板按钮 -->
      <button
        v-if="translateMode === 'multimodal'"
        class="btn"
        @click="$emit('openTransPanel')"
      >
        译文面板
      </button>
    </template>
  </div>
</template>

<script setup lang="ts">
/** 翻译状态类型 */
type TranslateStatus = 'idle' | 'translating' | 'done' | 'error'

/** 翻译模式类型 */
type TranslateMode = 'ocr' | 'multimodal'

defineProps<{
  /** 翻译状态 */
  translateStatus: TranslateStatus
  /** 翻译模式 */
  translateMode: TranslateMode
  /** 是否显示原文 */
  showOriginal: boolean
  /** 是否有翻译结果 */
  hasTranslation: boolean
}>()

defineEmits<{
  /** 翻译按钮点击 */
  translate: []
  /** 复制全部按钮点击 */
  copyAll: []
  /** 原文/译文切换按钮点击 */
  toggleOriginal: []
  /** 译文面板按钮点击 */
  openTransPanel: []
}>()
</script>

<style scoped>
.control-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
  background: transparent;
  min-height: var(--control-bar-height);
}

.btn {
  padding: 4px 12px;
  border: none;
  border-radius: var(--border-radius);
  background: rgba(255, 255, 255, 0.15);
  color: var(--color-text-primary);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s;
}

.btn:hover {
  background: rgba(255, 255, 255, 0.25);
}

.btn-primary {
  background: var(--color-accent);
  color: #000;
}

.btn-primary:hover {
  background: var(--color-accent-hover);
}
</style>

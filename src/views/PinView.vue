<template>
  <div
    class="pin-container"
    @mousedown="onMouseDown"
    @mousemove="onMouseMove"
    @mouseup="onMouseUp"
    @dblclick="onDoubleClick"
  >
    <div class="image-area" ref="imageArea">
      <img
        v-if="imageDataUrl"
        :src="imageDataUrl"
        class="pin-image"
        draggable="false"
        @load="onImageLoad"
      />
    </div>
    <ControlBar
      :translate-status="translateStatus"
      :translate-mode="translateMode"
      :show-original="showOriginal"
      :has-translation="hasTranslation"
      @translate="onTranslate"
      @copy-all="onCopyAll"
      @toggle-original="onToggleOriginal"
      @open-trans-panel="onOpenTransPanel"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { LogicalSize } from '@tauri-apps/api/dpi'
import { getPinImage } from '@/utils/tauri'
import { logger } from '@/utils/logger'
import ControlBar from '@/components/ControlBar.vue'

const TAG = 'PinView'

// 阴影内边距，需与后端 window/mod.rs 中的 PIN_PADDING 保持一致
const PIN_PADDING = 4

type TranslateStatus = 'idle' | 'translating' | 'done' | 'error'
type TranslateMode = 'ocr' | 'multimodal'

const imageDataUrl = ref<string>('')
const pinId = ref<string>('')
const translateStatus = ref<TranslateStatus>('idle')
const translateMode = ref<TranslateMode>('ocr')
const showOriginal = ref(false)
const hasTranslation = ref(false)

const imageArea = ref<HTMLElement | null>(null)

let mouseDownX = 0
let mouseDownY = 0
let hasStartedDrag = false

async function onImageLoad(event: Event) {
  const img = event.target as HTMLImageElement
  if (!img || !img.naturalWidth || !img.naturalHeight) return

  const dpr = window.devicePixelRatio || 1
  const logicalW = img.naturalWidth / dpr
  const logicalH = img.naturalHeight / dpr
  const controlBarH = 36

  logger.info(TAG, `图片加载完成: naturalSize=${img.naturalWidth}x${img.naturalHeight}, dpr=${dpr}, logicalSize=${logicalW}x${logicalH}`)

  try {
    await getCurrentWindow().setSize(new LogicalSize(
      logicalW + PIN_PADDING * 2,
      logicalH + controlBarH + PIN_PADDING * 2
    ))
    logger.info(TAG, `窗口大小调整成功: ${logicalW + PIN_PADDING * 2}x${logicalH + controlBarH + PIN_PADDING * 2}`)
  } catch (err) {
    logger.error(TAG, `窗口大小调整失败: ${err}`, err)
  }
}

onMounted(async () => {
  const currentWindow = getCurrentWindow()
  pinId.value = currentWindow.label
  logger.info(TAG, `PinView onMounted, windowLabel=${pinId.value}`)

  try {
    logger.info(TAG, `调用 getPinImage, windowId=${pinId.value}`)
    const base64Data = await getPinImage(pinId.value)
    if (base64Data) {
      logger.info(TAG, `获取到图片数据，长度=${base64Data.length}, startsWithData=${base64Data.startsWith('data:')}`)
      if (base64Data.startsWith('data:')) {
        imageDataUrl.value = base64Data
      } else {
        imageDataUrl.value = `data:image/png;base64,${base64Data}`
      }
      logger.info(TAG, `imageDataUrl 已设置，长度=${imageDataUrl.value.length}`)
    } else {
      logger.error(TAG, 'getPinImage 返回 null！图片数据未找到')
    }
  } catch (err) {
    logger.error(TAG, `getPinImage 调用失败: ${err}`, err)
  }
})

function onMouseDown(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (target.closest('.control-bar button')) return

  mouseDownX = e.clientX
  mouseDownY = e.clientY
  hasStartedDrag = false
}

async function onMouseMove(e: MouseEvent) {
  if (mouseDownX === 0 && mouseDownY === 0) return
  if (hasStartedDrag) return

  const dx = e.clientX - mouseDownX
  const dy = e.clientY - mouseDownY

  if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
    hasStartedDrag = true
    try {
      await getCurrentWindow().startDragging()
    } catch (err) {
      logger.error(TAG, `startDragging 失败: ${err}`, err)
    }
  }
}

function onMouseUp() {
  mouseDownX = 0
  mouseDownY = 0
  hasStartedDrag = false
}

async function onDoubleClick(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (imageArea.value && imageArea.value.contains(target)) {
    try {
      logger.info(TAG, '双击关闭贴图窗口')
      await getCurrentWindow().destroy()
    } catch (err) {
      logger.error(TAG, `双击关闭失败: ${err}`, err)
    }
  }
}

function onTranslate() {
  translateStatus.value = 'translating'
  setTimeout(() => {
    translateStatus.value = 'idle'
  }, 1500)
}

function onCopyAll() {
}

function onToggleOriginal() {
  showOriginal.value = !showOriginal.value
}

function onOpenTransPanel() {
}
</script>

<style scoped>
.pin-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  padding: 4px;
  background: transparent;
  user-select: none;
}

.image-area {
  flex: 1;
  overflow: hidden;
  position: relative;
  display: flex;
  align-items: flex-start;
  justify-content: flex-start;
  box-shadow: 0 1px 4px 0 rgba(0, 0, 0, 0.35);
}

.pin-image {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: fill;
  pointer-events: none;
}
</style>

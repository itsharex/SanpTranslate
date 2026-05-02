import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getConfig, saveConfig, type AppConfig } from '@/utils/tauri'

/** 应用配置状态管理 */
export const useConfigStore = defineStore('config', () => {
  /** 当前应用配置 */
  const config = ref<AppConfig | null>(null)
  /** 是否正在加载 */
  const loading = ref(false)
  /** 错误信息 */
  const error = ref<string | null>(null)

  /** 从后端加载配置 */
  async function loadConfig() {
    loading.value = true
    error.value = null
    try {
      config.value = await getConfig()
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  /** 更新并保存配置到后端 */
  async function updateConfig(newConfig: AppConfig) {
    loading.value = true
    error.value = null
    try {
      await saveConfig(newConfig)
      config.value = newConfig
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  return { config, loading, error, loadConfig, updateConfig }
})

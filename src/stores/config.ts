import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Config } from '../types'

export const useConfigStore = defineStore('config', () => {
  const config = ref<Config | null>(null)
  const loading = ref(false)

  async function load() {
    loading.value = true
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      config.value = await invoke<Config>('cmd_config_get')
    } finally {
      loading.value = false
    }
  }

  async function save(partial: Partial<Config>) {
    if (config.value) {
      Object.assign(config.value, partial)
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('cmd_config_set', { config: config.value })
    }
  }

  return { config, loading, load, save }
})

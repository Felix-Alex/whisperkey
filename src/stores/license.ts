import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useLicenseStore = defineStore('license', () => {
  const activated = ref(false)
  const loading = ref(false)

  async function refresh() {
    const { invoke } = await import('@tauri-apps/api/core')
    const result = await invoke<{ activated: boolean }>('cmd_license_status')
    activated.value = result.activated
  }

  async function activate(code: string) {
    loading.value = true
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const result = await invoke<{ activated: boolean }>('cmd_license_activate', { code })
      activated.value = result.activated
    } finally {
      loading.value = false
    }
  }

  return { activated, loading, refresh, activate }
})

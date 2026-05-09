import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { LicenseStatus } from '../types'

export const useLicenseStore = defineStore('license', () => {
  const status = ref<LicenseStatus>({
    activated: false,
    products: [],
  })

  async function refresh() {
    const { invoke } = await import('@tauri-apps/api/core')
    status.value = await invoke<LicenseStatus>('cmd_license_status')
  }

  async function activate(code: string) {
    const { invoke } = await import('@tauri-apps/api/core')
    status.value = await invoke<LicenseStatus>('cmd_license_activate', { code })
  }

  function isUnlocked(product: string): boolean {
    return status.value.products.includes(product)
  }

  return { status, refresh, activate, isUnlocked }
})

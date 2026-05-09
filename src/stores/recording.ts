import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { RecordingState } from '../types'

export const useRecordingStore = defineStore('recording', () => {
  const state = ref<RecordingState>({
    isRecording: false,
    elapsedMs: 0,
    level: 0,
    mode: 'raw',
  })

  async function toggle() {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('cmd_recording_toggle')
  }

  async function refresh() {
    const { invoke } = await import('@tauri-apps/api/core')
    state.value = await invoke<RecordingState>('cmd_recording_get_state')
  }

  return { state, toggle, refresh }
})

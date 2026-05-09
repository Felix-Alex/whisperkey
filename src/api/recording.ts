import { invoke } from '@tauri-apps/api/core'
import type { RecordingState } from '../types'

export async function toggleRecording(): Promise<void> {
  return invoke('cmd_recording_toggle')
}

export async function getRecordingState(): Promise<RecordingState> {
  return invoke<RecordingState>('cmd_recording_get_state')
}

import { invoke } from '@tauri-apps/api/core'
import type { HistoryItem } from '../types'

export async function listHistory(
  mode?: string,
  search?: string,
  page?: number,
): Promise<HistoryItem[]> {
  return invoke<HistoryItem[]>('cmd_history_list', { mode, search, page })
}

export async function deleteHistory(id: number): Promise<void> {
  return invoke('cmd_history_delete', { id })
}

export async function clearHistory(): Promise<void> {
  return invoke('cmd_history_clear')
}

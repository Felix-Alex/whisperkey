<script setup lang="ts">
import { ref, onMounted } from 'vue'
import type { HistoryItem } from '../../types'

const items = ref<HistoryItem[]>([])

onMounted(async () => {
  const { invoke } = await import('@tauri-apps/api/core')
  try {
    items.value = await invoke<HistoryItem[]>('cmd_history_list', { mode: null, search: null, page: 0 })
  } catch {
    items.value = []
  }
})

function formatTime(ts: number) {
  return new Date(ts * 1000).toLocaleString('zh-CN')
}

async function deleteItem(id: number) {
  const { invoke } = await import('@tauri-apps/api/core')
  await invoke('cmd_history_delete', { id })
  items.value = items.value.filter(i => i.id !== id)
}

async function clearAll() {
  const { invoke } = await import('@tauri-apps/api/core')
  await invoke('cmd_history_clear')
  items.value = []
}
</script>

<template>
  <div class="settings-page">
    <div class="history-header">
      <h3>历史记录</h3>
      <button class="btn-danger" @click="clearAll">清空全部</button>
    </div>

    <div class="history-list" v-if="items.length">
      <div v-for="item in items" :key="item.id" class="history-item">
        <div class="item-meta">
          <span class="item-mode">{{ item.mode }}</span>
          <span class="item-time">{{ formatTime(item.createdAt) }}</span>
          <button class="btn-small" @click="deleteItem(item.id)">删除</button>
        </div>
        <p class="item-text">{{ item.processedText || item.rawText }}</p>
      </div>
    </div>
    <p v-else class="empty">暂无记录</p>
  </div>
</template>

<style scoped>
.settings-page h3 { margin-bottom: 16px; }
.history-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.history-item { padding: 12px; border-bottom: 1px solid #eee; }
.item-meta { display: flex; gap: 12px; align-items: center; margin-bottom: 4px; font-size: 12px; color: #666; }
.item-mode { background: #e8f0fe; color: #1a73e8; padding: 2px 8px; border-radius: 4px; font-weight: 500; }
.item-text { font-size: 14px; color: #333; }
.empty { color: #999; text-align: center; padding: 32px; }
.btn-danger { padding: 6px 12px; background: #d32f2f; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 12px; }
.btn-small { padding: 2px 8px; background: #eee; border: none; border-radius: 4px; cursor: pointer; font-size: 11px; }
</style>

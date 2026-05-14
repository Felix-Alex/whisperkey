<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NButton, NText, NPopconfirm } from 'naive-ui'
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

const modeLabels: Record<string, string> = {
  raw: '原话', polish: '优化', markdown: 'Markdown', quick_ask: '速问', custom: '自定义',
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
  <div>
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px">
      <h3 style="font-size: 16px; font-weight: 500; margin: 0">历史记录</h3>
      <NPopconfirm @positive-click="clearAll">
        <template #trigger>
          <NButton size="small" type="error" text>清空全部</NButton>
        </template>
        确认清空全部历史记录？
      </NPopconfirm>
    </div>

    <div v-if="items.length">
      <div v-for="item in items" :key="item.id" style="padding: 12px 0; border-bottom: 1px solid var(--border-color)">
        <div style="display: flex; gap: 8px; align-items: center; margin-bottom: 4px">
          <NText tag="span" depth="3" style="font-size: 11px; background: var(--color-action); padding: 1px 6px; border-radius: 3px">
            {{ modeLabels[item.mode] || item.mode }}
          </NText>
          <NText depth="3" style="font-size: 11px">{{ formatTime(item.createdAt) }}</NText>
          <NButton size="tiny" text @click="deleteItem(item.id)">删除</NButton>
        </div>
        <NText style="font-size: 13px; line-height: 1.5">{{ item.processedText || item.rawText }}</NText>
      </div>
    </div>
    <NText v-else depth="3" style="text-align: center; padding: 32px 0; display: block">暂无记录</NText>
  </div>
</template>

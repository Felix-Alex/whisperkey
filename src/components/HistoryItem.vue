<script setup lang="ts">
import type { HistoryItem } from '../types'

const { item } = defineProps<{ item: HistoryItem }>()
const emit = defineEmits<{
  delete: [id: number]
  reinject: [id: number]
}>()

function formatTime(ts: number) {
  return new Date(ts * 1000).toLocaleString('zh-CN')
}
</script>

<template>
  <div class="item">
    <div class="item-meta">
      <span class="item-mode">{{ item.mode }}</span>
      <span class="item-time">{{ formatTime(item.createdAt) }}</span>
      <span v-if="item.appName" class="item-app">{{ item.appName }}</span>
      <span class="item-injected">{{ item.injected ? '已注入' : '' }}</span>
    </div>
    <p class="item-text">{{ item.processedText || item.rawText }}</p>
    <div class="item-actions">
      <button @click="emit('reinject', item.id)">重新注入</button>
      <button @click="emit('delete', item.id)">删除</button>
    </div>
  </div>
</template>

<style scoped>
.item {
  padding: 12px;
  border-bottom: 1px solid #eee;
}
.item-meta { display: flex; gap: 12px; align-items: center; margin-bottom: 4px; font-size: 12px; color: #666; }
.item-mode { background: #e8f0fe; color: #1a73e8; padding: 1px 6px; border-radius: 4px; }
.item-app { color: #999; }
.item-injected { color: #2e7d32; }
.item-text { font-size: 14px; margin: 4px 0; }
.item-actions { display: flex; gap: 8px; }
button { padding: 2px 8px; background: #eee; border: none; border-radius: 4px; cursor: pointer; font-size: 11px; }
button:hover { background: #ddd; }
</style>

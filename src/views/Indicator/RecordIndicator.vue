<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import type { IndicatorState } from '../../types'
import { listen } from '@tauri-apps/api/event'

const state = ref<IndicatorState>('Idle')
const level = ref(0)
const mode = ref('raw')
const partialText = ref('')

let unlisten: (() => void) | null = null
let unlistenText: (() => void) | null = null

onMounted(async () => {
  unlisten = await listen<{ state: IndicatorState; level: number; mode: string }>('indicator://state', (event) => {
    state.value = event.payload.state
    level.value = event.payload.level
    mode.value = event.payload.mode
  })
  unlistenText = await listen<{ text: string; final?: boolean }>('asr-partial-text', (event) => {
    partialText.value = event.payload.text
  })
})

onUnmounted(() => {
  unlisten?.()
  unlistenText?.()
})
</script>

<template>
  <div class="indicator" :class="{ recording: state === 'Recording', processing: state === 'Processing' }">
    <div class="indicator-dot" :class="{ active: state !== 'Idle' }"></div>
    <div class="indicator-level" v-if="state === 'Recording'">
      <div class="level-bar" v-for="i in 12" :key="i" :style="{ height: (level * 100 * Math.random()).toFixed(0) + '%' }"></div>
    </div>
    <div class="indicator-spinner" v-if="state === 'Processing'"></div>
    <span class="indicator-mode">{{ mode }}</span>
    <span class="indicator-text" v-if="partialText && state === 'Recording'">{{ partialText }}</span>
  </div>
</template>

<style scoped>
.indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  height: 40px;
  border-radius: 20px;
  background: rgba(0, 0, 0, 0.75);
  color: #fff;
  font-size: 12px;
  user-select: none;
  -webkit-app-region: drag;
}
.indicator-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #666;
  flex-shrink: 0;
}
.indicator-dot.active {
  background: #ff4444;
  animation: pulse 1s ease-in-out infinite;
}
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}
.indicator-level {
  display: flex;
  align-items: flex-end;
  gap: 2px;
  height: 20px;
}
.level-bar {
  width: 3px;
  background: #4caf50;
  border-radius: 1px;
  transition: height 0.1s;
}
.indicator-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255,255,255,0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
.indicator-mode {
  font-weight: 500;
}
.indicator-text {
  max-width: 320px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #ccc;
  font-size: 11px;
  margin-left: 4px;
}
</style>

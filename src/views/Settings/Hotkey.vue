<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useConfigStore } from '../../stores/config'

const store = useConfigStore()
const hotkeyStr = ref('Ctrl+Shift+Space')

onMounted(async () => {
  await store.load()
  if (store.config) {
    hotkeyStr.value = store.config.hotkey.modifiers.join('+') + '+' + store.config.hotkey.key
  }
})

async function saveHotkey() {
  const parts = hotkeyStr.value.split('+').map(s => s.trim())
  const key = parts.pop() || 'Space'
  await store.save({ hotkey: { modifiers: parts, key, paused: false } } as any)
}
</script>

<template>
  <div class="settings-page">
    <h3>快捷键设置</h3>
    <div class="setting-group">
      <label class="setting-label">快捷键</label>
      <input
        v-model="hotkeyStr"
        type="text"
        placeholder="例如: Ctrl+Shift+Space"
        @change="saveHotkey"
      />
    </div>
    <p class="hint">支持: Ctrl, Shift, Alt, Win + 字母/数字/F键/特殊键</p>
  </div>
</template>

<style scoped>
.settings-page h3 { margin-bottom: 16px; }
.setting-group { margin-bottom: 12px; }
.setting-label { display: block; font-size: 14px; margin-bottom: 4px; }
input[type="text"] {
  width: 240px;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}
.hint { font-size: 12px; color: #888; margin-top: 8px; }
</style>

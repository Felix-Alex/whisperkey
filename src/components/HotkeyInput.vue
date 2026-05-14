<script setup lang="ts">
import { ref } from 'vue'

const model = defineModel<string>({ default: 'Alt+J' })
const capturing = ref(false)
const errorMsg = ref('')

const MODIFIER_KEYS = ['Control', 'Shift', 'Alt', 'Meta']

function startCapture() {
  capturing.value = true
  errorMsg.value = ''
}

function onKeyDown(e: KeyboardEvent) {
  if (!capturing.value) return
  e.preventDefault()

  if (e.key === 'Escape') {
    cancelCapture()
    return
  }

  // Ignore modifier-only keydowns
  if (MODIFIER_KEYS.includes(e.key)) return

  const keys: string[] = []
  if (e.ctrlKey) keys.push('Ctrl')
  if (e.shiftKey) keys.push('Shift')
  if (e.altKey) keys.push('Alt')
  if (e.metaKey) keys.push('Win')

  const keyName = e.key === ' ' ? 'Space' : e.key.length === 1 ? e.key.toUpperCase() : e.key
  keys.push(keyName)

  const totalKeys = keys.length
  if (totalKeys < 2) {
    errorMsg.value = '必须包含至少一个修饰键（Ctrl/Alt/Shift/Win）'
    return
  }
  if (totalKeys > 3) {
    errorMsg.value = '最多支持 3 键组合'
    return
  }

  errorMsg.value = ''
  model.value = keys.join('+')
  capturing.value = false
}

function cancelCapture() {
  capturing.value = false
  errorMsg.value = ''
}

function onBlur() {
  capturing.value = false
  errorMsg.value = ''
}
</script>

<template>
  <div>
    <input
      :value="capturing ? '请按下快捷键...' : (model || 'Alt+J')"
      type="text"
      readonly
      :class="['hotkey-input', { capturing }]"
      @focus="startCapture"
      @blur="onBlur"
      @keydown="onKeyDown"
    />
    <p v-if="errorMsg" class="error-text">{{ errorMsg }}</p>
    <p v-else class="hint-text">点击输入框后按下快捷键组合（2-3 键，至少一个修饰键）</p>
  </div>
</template>

<style scoped>
.hotkey-input {
  width: 260px;
  padding: 8px 14px;
  border: 1px solid var(--border-color, #ccc);
  border-radius: 6px;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  text-align: center;
  background: var(--input-color, #fff);
  color: var(--text-color, #333);
  transition: border-color 0.15s, background 0.15s;
}
.hotkey-input.capturing {
  border-color: var(--primary-color, #1a73e8);
  background: var(--primary-color-suppl, #e8f0fe);
}
.hotkey-input:focus {
  outline: none;
}
.error-text {
  font-size: 12px;
  color: #e53935;
  margin-top: 4px;
}
.hint-text {
  font-size: 11px;
  color: #999;
  margin-top: 4px;
}
</style>

<script setup lang="ts">
import { ref } from 'vue'

const model = defineModel<string>()
const isFocused = ref(false)

function onKeyDown(e: KeyboardEvent) {
  if (!isFocused.value) return
  e.preventDefault()

  const keys: string[] = []
  if (e.ctrlKey) keys.push('Ctrl')
  if (e.shiftKey) keys.push('Shift')
  if (e.altKey) keys.push('Alt')
  if (e.metaKey) keys.push('Win')

  const keyName = e.key === ' ' ? 'Space' : e.key.length === 1 ? e.key.toUpperCase() : e.key
  if (!['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
    keys.push(keyName)
  }

  if (keys.length >= 2) {
    model.value = keys.join('+')
  }
}
</script>

<template>
  <input
    :value="model"
    type="text"
    readonly
    placeholder="点击后按下快捷键"
    @focus="isFocused = true"
    @blur="isFocused = false"
    @keydown="onKeyDown"
  />
</template>

<style scoped>
input {
  width: 220px;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
}
input:focus {
  border-color: #1a73e8;
  outline: none;
}
</style>

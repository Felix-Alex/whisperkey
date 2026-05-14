<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { NButton, NText, NSpace, useMessage } from 'naive-ui'
import HotkeyInput from '../../components/HotkeyInput.vue'
import { useConfigStore } from '../../stores/config'

const store = useConfigStore()
const message = useMessage()
const hotkeyStr = ref('Alt+J')
const dirty = ref(false)

onBeforeRouteLeave((_to, _from, next) => {
  if (dirty.value) {
    const answer = window.confirm('有未保存的快捷键更改，确定离开吗？')
    if (!answer) return next(false)
    dirty.value = false
  }
  next()
})

onMounted(async () => {
  await store.load()
  if (store.config) {
    hotkeyStr.value = store.config.hotkey.modifiers.join('+') + '+' + store.config.hotkey.key
  }
})

async function saveHotkey() {
  const parts = hotkeyStr.value.split('+').map(s => s.trim())
  const key = parts.pop() || 'J'
  const modifiers = parts
  if (modifiers.length === 0) {
    message.error('快捷键必须包含至少一个修饰键（Ctrl/Alt/Shift/Win）')
    return
  }
  if (modifiers.length + 1 > 3) {
    message.error('最多支持 3 键组合')
    return
  }

  try {
    if (!store.config) return
    store.config.hotkey.modifiers = modifiers
    store.config.hotkey.key = key
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('cmd_config_set', { config: store.config })
    await invoke('cmd_config_save')
    await invoke('cmd_hotkey_restart')
    dirty.value = false
    message.success('快捷键已保存并生效')
  } catch (e: any) {
    message.error(typeof e === 'string' ? e : '保存失败')
  }
}
</script>

<template>
  <div>
    <h3 style="margin-bottom: 16px; font-size: 16px; font-weight: 500">快捷键设置</h3>

    <NSpace vertical :size="12">
      <NText depth="2">当前快捷键组合</NText>
      <HotkeyInput v-model="hotkeyStr" @update:model-value="dirty = true" />
      <NButton type="primary" @click="saveHotkey">保存快捷键</NButton>
    </NSpace>
  </div>
</template>

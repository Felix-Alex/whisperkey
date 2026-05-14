<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { NButton, NInputNumber, NSpace, NText, NDivider, useMessage } from 'naive-ui'
import { useConfigStore } from '../../stores/config'
import { useLicenseStore } from '../../stores/license'
import type { OutputMode } from '../../types'

const store = useConfigStore()
const licenseStore = useLicenseStore()
const message = useMessage()

const outputMode = ref<OutputMode>('raw')
const maxDuration = ref(60)
const customPromptContent = ref('')
const customPromptSaving = ref(false)
const showCustomEditor = ref(false)
const dirty = ref(false)

onBeforeRouteLeave((_to, _from, next) => {
  if (dirty.value) {
    const answer = window.confirm('有未保存的更改，确定离开吗？')
    if (!answer) return next(false)
    dirty.value = false
  }
  next()
})

function openCustomEditor() {
  showCustomEditor.value = true
  loadCustomPrompt()
}

const modeOptions = [
  { value: 'raw', label: '原话模式', desc: '语音直接转文字，绝对不经过 LLM' },
  { value: 'polish', label: '优化模式', desc: 'AI 去除口语词，优化为书面语言' },
  { value: 'quick_ask', label: '速问模式', desc: '语音提问，AI 快速给出答案' },
  { value: 'markdown', label: 'Markdown 模式', desc: '将口述需求转为结构化 AI 提示词' },
  { value: 'custom', label: '自定义模式', desc: '使用自定义 prompt 处理语音转文字结果' },
]

const isLocked = (mode: string) => mode !== 'raw' && !licenseStore.activated

onMounted(async () => {
  await licenseStore.refresh()
  await store.load()
  if (store.config) {
    outputMode.value = store.config.outputMode || 'raw'
    maxDuration.value = store.config.audio.maxDurationSec
  }
})

async function selectMode(mode: OutputMode) {
  if (isLocked(mode)) return
  outputMode.value = mode
  if (store.config) { store.config.outputMode = mode }
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('cmd_set_output_mode', { mode })
    if (store.config) {
      await invoke('cmd_config_set', { config: store.config })
      await invoke('cmd_config_save')
    }
    message.success(`默认输出模式已切换为：${modeOptions.find(m => m.value === mode)?.label}`)
  } catch (e: any) {
    message.error(typeof e === 'string' ? e : '保存失败')
  }
}

async function loadCustomPrompt() {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    customPromptContent.value = await invoke<string>('cmd_custom_prompt_get')
  } catch (e: any) {
    message.error(typeof e === 'string' ? e : '加载失败')
  }
}

async function saveCustomPrompt() {
  if (customPromptSaving.value) return
  customPromptSaving.value = true
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('cmd_custom_prompt_set', { content: customPromptContent.value })
    showCustomEditor.value = false
    message.success('自定义 Prompt 已保存，即时生效')
  } catch (e: any) {
    message.error(typeof e === 'string' ? e : '保存失败')
  } finally {
    customPromptSaving.value = false
  }
}

async function saveConfig() {
  if (!store.config) return
  try {
    store.config.outputMode = outputMode.value
    store.config.audio.maxDurationSec = maxDuration.value
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('cmd_config_set', { config: store.config })
    await invoke('cmd_config_save')
    dirty.value = false
    message.success('设置已保存')
  } catch (e: any) {
    message.error(typeof e === 'string' ? e : '保存失败')
  }
}
</script>

<template>
  <div>
    <h3 style="margin-bottom: 16px; font-size: 16px; font-weight: 500">输出模式</h3>

    <NSpace vertical :size="8">
      <div
        v-for="m in modeOptions"
        :key="m.value"
        :style="{
          padding: '14px 16px',
          border: outputMode === m.value ? '2px solid #2080f0' : '1px solid rgba(128,128,128,0.22)',
          borderRadius: '8px',
          cursor: isLocked(m.value) ? 'not-allowed' : 'pointer',
          opacity: isLocked(m.value) ? 0.45 : 1,
          background: outputMode === m.value ? 'rgba(32,128,240,0.06)' : 'transparent',
          transition: 'all 0.15s ease',
        }"
        @click="selectMode(m.value as OutputMode)"
      >
        <div style="display: flex; align-items: center; justify-content: space-between">
          <div>
            <NText strong :type="outputMode === m.value ? 'primary' : 'default'">{{ m.label }}</NText>
            <NText depth="3" tag="div" style="font-size: 12px; margin-top: 2px">{{ m.desc }}</NText>
          </div>
          <span v-if="outputMode === m.value" style="color: #2080f0; font-size: 18px">&#10003;</span>
          <NText v-else-if="isLocked(m.value)" type="error" style="font-size: 12px">需激活后使用</NText>
        </div>
      </div>
    </NSpace>

    <NButton
      v-if="outputMode === 'custom' && !showCustomEditor"
      size="small"
      style="margin-top: 10px"
      @click="openCustomEditor"
    >编辑 Prompt</NButton>

    <div
      v-if="showCustomEditor"
      style="margin-top: 12px; padding: 12px; border: 1px solid rgba(128,128,128,0.18); border-radius: 6px; background: rgba(0,0,0,0.02)"
    >
      <textarea
        v-model="customPromptContent"
        rows="6"
        style="width: 100%; padding: 10px; border: 1px solid rgba(128,128,128,0.22); border-radius: 4px; font-size: 13px; line-height: 1.6; resize: vertical; font-family: inherit; outline: none; box-sizing: border-box"
        placeholder="输入自定义 prompt 指令..."
      />
      <div style="display: flex; align-items: center; gap: 10px; margin-top: 8px">
        <NButton size="small" type="primary" :loading="customPromptSaving" @click="saveCustomPrompt">保存</NButton>
        <NText depth="3" style="font-size: 11px">保存后即时生效，无需重启</NText>
      </div>
    </div>

    <NDivider />

    <h3 style="margin-bottom: 12px; font-size: 16px; font-weight: 500">录音设置</h3>

    <NSpace vertical :size="12">
      <div style="display: flex; align-items: center; gap: 12px">
        <NText>录音最大时长（秒）</NText>
        <NInputNumber v-model:value="maxDuration" :min="10" :max="120" style="width: 80px" @update:value="dirty = true" />
      </div>
      <NText depth="3" style="font-size: 12px">静音 5 秒后自动结束录音</NText>
    </NSpace>

    <NDivider />

    <NButton type="primary" @click="saveConfig">保存设置</NButton>
  </div>
</template>

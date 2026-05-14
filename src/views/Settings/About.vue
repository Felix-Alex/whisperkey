<script setup lang="ts">
import { NButton, NText, NSpace, NDivider } from 'naive-ui'

const version = '2.1.6'

async function openLogs() {
  const { invoke } = await import('@tauri-apps/api/core')
  await invoke('cmd_app_open_logs_folder')
}
</script>

<template>
  <div>
    <h3 style="margin-bottom: 16px; font-size: 16px; font-weight: 500">关于 WhisperKey</h3>

    <NSpace vertical :size="8">
      <NText>版本：{{ version }}</NText>
      <NText depth="2" style="font-size: 13px">开发者：SuperAlex (hfli_07@126.com)</NText>
      <NText depth="2" style="font-size: 13px">技术栈：Tauri 2 + Vue 3 + Naive UI + Rust</NText>
      <NText depth="2" style="font-size: 13px">五种输出模式：原话 / 优化 / 速问 / Markdown / 自定义</NText>
    </NSpace>

    <NDivider />

    <NButton @click="openLogs">打开日志文件夹</NButton>
  </div>
</template>

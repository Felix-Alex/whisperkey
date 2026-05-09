<script setup lang="ts">
const version = '0.1.0'

async function openLogs() {
  const { invoke } = await import('@tauri-apps/api/core')
  await invoke('cmd_app_open_logs_folder')
}
</script>

<template>
  <div class="settings-page">
    <h3>关于 WhisperKey</h3>
    <div class="about-info">
      <p><strong>版本：</strong>{{ version }}</p>
      <p><strong>技术栈：</strong>Tauri 2 + Vue 3 + Rust</p>
      <button @click="openLogs">打开日志文件夹</button>
    </div>
  </div>
</template>

<style scoped>
.settings-page h3 { margin-bottom: 16px; }
.about-info p { margin-bottom: 8px; font-size: 14px; }
button {
  margin-top: 12px;
  padding: 8px 16px;
  background: #1a73e8;
  color: #fff;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}
button:hover { background: #1557b0; }
</style>

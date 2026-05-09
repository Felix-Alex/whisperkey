<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useConfigStore } from '../../stores/config'

const store = useConfigStore()
const openaiKey = ref('')

onMounted(async () => {
  await store.load()
  if (store.config) {
    openaiKey.value = store.config.providers.openai.apiKey || ''
  }
})
</script>

<template>
  <div class="settings-page">
    <h3>服务商配置</h3>

    <div class="provider-card">
      <h4>ASR - 语音识别</h4>
      <div class="setting-group">
        <label class="setting-label">OpenAI API Key</label>
        <input v-model="openaiKey" type="password" placeholder="sk-..." />
      </div>
    </div>

    <div class="provider-card">
      <h4>LLM - 文本优化</h4>
      <p class="hint">模式分配：原话=官方免费 / 优化=DeepSeek / Markdown=Anthropic</p>
    </div>
  </div>
</template>

<style scoped>
.settings-page h3 { margin-bottom: 16px; }
.provider-card {
  background: #fff;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 12px;
}
.provider-card h4 { margin-bottom: 8px; font-size: 14px; }
.setting-group { margin-bottom: 8px; }
.setting-label { display: block; font-size: 13px; margin-bottom: 4px; }
input[type="password"] {
  width: 320px;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}
.hint { font-size: 12px; color: #888; }
</style>

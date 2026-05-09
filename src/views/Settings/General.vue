<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useConfigStore } from '../../stores/config'

const store = useConfigStore()
const maxDuration = ref(60)
const silenceAutoStop = ref(false)
const silenceTimeoutMs = ref(3000)

onMounted(async () => {
  await store.load()
  if (store.config) {
    maxDuration.value = store.config.audio.maxDurationSec
    silenceAutoStop.value = store.config.audio.silenceAutoStop
    silenceTimeoutMs.value = store.config.audio.silenceTimeoutMs
  }
})

async function saveAudio() {
  await store.save({
    audio: {
      maxDurationSec: maxDuration.value,
      silenceAutoStop: silenceAutoStop.value,
      silenceTimeoutMs: silenceTimeoutMs.value,
      inputDevice: 'default',
    },
  } as any)
}
</script>

<template>
  <div class="settings-page">
    <h3>通用设置</h3>

    <div class="setting-group">
      <label class="setting-label">录音最大时长（秒）</label>
      <input v-model.number="maxDuration" type="number" min="10" max="120" @change="saveAudio" />
    </div>

    <div class="setting-group">
      <label class="setting-label">
        <input v-model="silenceAutoStop" type="checkbox" @change="saveAudio" />
        静音自动停止
      </label>
    </div>

    <div v-if="silenceAutoStop" class="setting-group">
      <label class="setting-label">静音超时（毫秒）</label>
      <input v-model.number="silenceTimeoutMs" type="number" min="1000" max="10000" step="500" @change="saveAudio" />
    </div>
  </div>
</template>

<style scoped>
.settings-page h3 {
  margin-bottom: 16px;
}
.setting-group {
  margin-bottom: 12px;
}
.setting-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}
input[type="number"] {
  width: 80px;
  padding: 4px 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
}
</style>

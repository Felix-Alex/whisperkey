<script setup lang="ts">
import { ref } from 'vue'
import { useLicenseStore } from '../../stores/license'

const licenseStore = useLicenseStore()
const code = ref('')
const message = ref('')

async function activate() {
  if (!code.value.trim()) return
  try {
    await licenseStore.activate(code.value.trim())
    message.value = '激活成功！'
  } catch {
    message.value = '激活失败，请检查激活码'
  }
}
</script>

<template>
  <div class="dialog">
    <h3>激活 WhisperKey</h3>
    <p>输入您的激活码以解锁 优化 和 Markdown 模式</p>
    <input v-model="code" type="text" placeholder="激活码" />
    <button @click="activate">激活</button>
    <p v-if="message" class="msg">{{ message }}</p>
  </div>
</template>

<style scoped>
.dialog {
  padding: 24px;
  text-align: center;
}
h3 { margin-bottom: 8px; }
p { font-size: 14px; color: #666; margin-bottom: 16px; }
input {
  width: 100%;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  margin-bottom: 12px;
  font-size: 14px;
}
button {
  padding: 10px 24px;
  background: #1a73e8;
  color: #fff;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}
.msg { margin-top: 12px; font-weight: 500; }
</style>

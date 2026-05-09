<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useLicenseStore } from '../../stores/license'

const licenseStore = useLicenseStore()
const code = ref('')

onMounted(() => licenseStore.refresh())

async function activate() {
  if (code.value.trim()) {
    await licenseStore.activate(code.value.trim())
    code.value = ''
  }
}
</script>

<template>
  <div class="settings-page">
    <h3>激活</h3>

    <div class="status-card">
      <p>
        当前状态：
        <span :class="licenseStore.status.activated ? 'active' : 'inactive'">
          {{ licenseStore.status.activated ? '已激活' : '未激活' }}
        </span>
      </p>
      <p v-if="licenseStore.status.products.length">
        已解锁：{{ licenseStore.status.products.join(', ') }}
      </p>
    </div>

    <div class="activate-form">
      <input v-model="code" type="text" placeholder="输入激活码" />
      <button @click="activate">激活</button>
    </div>
  </div>
</template>

<style scoped>
.settings-page h3 { margin-bottom: 16px; }
.status-card { margin-bottom: 16px; padding: 12px; background: #f9f9f9; border-radius: 8px; }
.active { color: #2e7d32; font-weight: 600; }
.inactive { color: #c62828; }
.activate-form { display: flex; gap: 8px; }
input[type="text"] {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}
button {
  padding: 8px 16px;
  background: #1a73e8;
  color: #fff;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}
button:hover { background: #1557b0; }
</style>

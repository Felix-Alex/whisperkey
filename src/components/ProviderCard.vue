<script setup lang="ts">
const { name, configured, testOk } = defineProps<{
  name: string
  configured: boolean
  testOk: boolean
}>()

const emit = defineEmits<{
  test: []
  config: []
}>()
</script>

<template>
  <div class="card">
    <div class="card-header">
      <span class="card-name">{{ name }}</span>
      <span class="card-status" :class="{ ok: testOk }">
        {{ testOk ? '✓ 已连接' : configured ? '已配置' : '未配置' }}
      </span>
    </div>
    <div class="card-actions">
      <button @click="emit('config')">配置</button>
      <button @click="emit('test')" :disabled="!configured">测试</button>
    </div>
  </div>
</template>

<style scoped>
.card {
  background: #fff;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 12px 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}
.card-header { display: flex; gap: 12px; align-items: center; }
.card-name { font-weight: 500; font-size: 14px; }
.card-status { font-size: 12px; color: #888; }
.card-status.ok { color: #2e7d32; }
.card-actions { display: flex; gap: 8px; }
button {
  padding: 4px 12px;
  background: #eee;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}
button:hover { background: #ddd; }
button:disabled { opacity: 0.5; cursor: default; }
</style>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const currentRoute = ref(router.currentRoute.value.path)

const navItems = [
  { path: '/settings/general', label: '通用', icon: '⚙' },
  { path: '/settings/hotkey', label: '快捷键', icon: '⌨' },
  { path: '/settings/providers', label: '服务商', icon: '☁' },
  { path: '/settings/activation', label: '激活', icon: '🔑' },
  { path: '/settings/history', label: '历史', icon: '📋' },
  { path: '/settings/about', label: '关于', icon: 'ℹ' },
]

function navigate(path: string) {
  currentRoute.value = path
  router.push(path)
}
</script>

<template>
  <div class="app-shell">
    <nav class="sidebar">
      <div class="sidebar-header">
        <h2>WhisperKey</h2>
      </div>
      <ul class="nav-list">
        <li
          v-for="item in navItems"
          :key="item.path"
          :class="{ active: currentRoute === item.path }"
          @click="navigate(item.path)"
        >
          <span class="nav-icon">{{ item.icon }}</span>
          <span class="nav-label">{{ item.label }}</span>
        </li>
      </ul>
    </nav>
    <main class="content">
      <router-view />
    </main>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
  background: #f5f5f5;
  color: #333;
  overflow: hidden;
}

.app-shell {
  display: flex;
  height: 100vh;
}

.sidebar {
  width: 180px;
  background: #fff;
  border-right: 1px solid #e0e0e0;
  display: flex;
  flex-direction: column;
  user-select: none;
}

.sidebar-header {
  padding: 16px;
  border-bottom: 1px solid #e0e0e0;
}

.sidebar-header h2 {
  font-size: 16px;
  font-weight: 600;
}

.nav-list {
  list-style: none;
  padding: 8px;
}

.nav-list li {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: background 0.15s;
}

.nav-list li:hover {
  background: #f0f0f0;
}

.nav-list li.active {
  background: #e8f0fe;
  color: #1a73e8;
}

.nav-icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
}

.content {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
}
</style>

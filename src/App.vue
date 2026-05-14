<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { darkTheme, zhCN, dateZhCN, NConfigProvider, NDialogProvider, NLayout, NLayoutSider, NMenu, NMessageProvider, NNotificationProvider, NText } from 'naive-ui'
import type { MenuOption } from 'naive-ui'

const router = useRouter()
const route = useRoute()
const activeKey = ref(route.path)

watch(() => route.path, (path) => {
  activeKey.value = path
})

const menuOptions: MenuOption[] = [
  { label: '通用', key: '/settings/general' },
  { label: '快捷键', key: '/settings/hotkey' },
  { label: '服务商', key: '/settings/providers' },
  { label: '激活', key: '/settings/activation' },
  { label: '历史', key: '/settings/history' },
  { label: '关于', key: '/settings/about' },
]

function handleMenuUpdate(key: string) {
  activeKey.value = key
  router.push(key)
}
</script>

<template>
  <NConfigProvider :theme="darkTheme" :locale="zhCN" :date-locale="dateZhCN">
    <NLayout has-sider style="height: 100vh">
      <NLayoutSider
        bordered
        collapse-mode="width"
        :collapsed-width="0"
        :width="180"
        :native-scrollbar="false"
      >
        <div style="padding: 16px 20px; text-align: center">
          <div style="display: inline-block">
            <NText tag="h3" type="primary" style="margin: 0; font-weight: 700; font-size: 20px; letter-spacing: 2px">
              语灵听写
            </NText>
            <NText tag="div" type="primary" depth="2" style="font-size: 10px; text-align: right">
              WhisperKey
            </NText>
          </div>
        </div>
        <NMenu
          :value="activeKey"
          :options="menuOptions"
          @update:value="handleMenuUpdate"
        />
      </NLayoutSider>
      <NLayout>
        <NMessageProvider>
          <NDialogProvider>
            <NNotificationProvider>
              <div style="padding: 32px 40px; max-width: 720px">
                <router-view />
              </div>
            </NNotificationProvider>
          </NDialogProvider>
        </NMessageProvider>
      </NLayout>
    </NLayout>
  </NConfigProvider>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}
body {
  font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
  overflow: hidden;
}
</style>

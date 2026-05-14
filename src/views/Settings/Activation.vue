<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NButton, NInput, NText, NSpace, useMessage } from 'naive-ui'
import { useLicenseStore } from '../../stores/license'

const licenseStore = useLicenseStore()
const message = useMessage()
const code = ref('')

onMounted(async () => {
  await licenseStore.refresh()
})

async function handleActivate() {
  if (!code.value.trim()) {
    message.warning('请输入激活码')
    return
  }
  try {
    await licenseStore.activate(code.value.trim())
    message.success('激活成功')
  } catch (e: any) {
    message.error(typeof e === 'string' ? e : '激活失败')
  }
}
</script>

<template>
  <div>
    <h3 style="margin-bottom: 16px; font-size: 16px; font-weight: 500">激活许可证</h3>

    <template v-if="licenseStore.activated">
      <NText type="success" style="font-size: 15px">
        恭喜您，解锁了所有的输出模式。
      </NText>
    </template>

    <template v-else>
      <NText depth="2" style="margin-bottom: 16px; display: block">
        未激活状态下，仅"原话模式"可用。
      </NText>

      <NSpace vertical :size="12" style="margin-top: 16px">
        <div style="display: flex; align-items: center; gap: 12px">
          <NInput
            v-model:value="code"
            maxlength="6"
            style="width: 180px; font-family: monospace; font-size: 18px; letter-spacing: 4px; text-align: center"
            placeholder=""
          />
          <NButton type="primary" :loading="licenseStore.loading" @click="handleActivate">
            激活
          </NButton>
        </div>
      </NSpace>
    </template>
  </div>
</template>

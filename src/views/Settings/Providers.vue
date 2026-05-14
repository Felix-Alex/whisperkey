<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { NButton, NSelect, NInput, NSpace, NText, NDivider, useMessage } from 'naive-ui'
import { useConfigStore } from '../../stores/config'

const store = useConfigStore()

function mask(len: number): string { return '•'.repeat(len) }

const message = useMessage()

const testingLlm = ref(false)
const testingAsr = ref(false)
const dirty = ref(false)

onBeforeRouteLeave((_to, _from, next) => {
  if (dirty.value) {
    const answer = window.confirm('有未保存的配置更改，确定离开吗？')
    if (!answer) return next(false)
    dirty.value = false
  }
  next()
})

// ── LLM state ──
const llmProvider = ref('openai')
const llmApiKey = ref('')
const llmSecretKey = ref('')
const llmBaseUrl = ref('https://api.openai.com/v1')
const llmModel = ref('gpt-4o-mini')
const llmKeyConfigured = ref(false)
const llmKeyFocused = ref(false)
const llmSecretConfigured = ref(false)
const llmSecretFocused = ref(false)

// ── ASR state ──
const asrProvider = ref('openai')
const asrApiKey = ref('')
const asrApiSecret = ref('')
const asrBaseUrl = ref('https://api.openai.com/v1')
const asrModel = ref('whisper-1')
const asrLanguage = ref('auto')
const asrKeyConfigured = ref(false)
const asrKeyFocused = ref(false)
const asrSecretConfigured = ref(false)
const asrSecretFocused = ref(false)

const llmProviderOptions = [
  { value: 'openai', label: 'OpenAI' }, { value: 'deepseek', label: 'DeepSeek' },
  { value: 'qwen', label: '通义千问' }, { value: 'doubao', label: '豆包' },
  { value: 'anthropic', label: 'Anthropic Claude' }, { value: 'gemini', label: 'Google Gemini' },
  { value: 'ernie', label: '百度文心一言' },
]

const asrProviderOptions = [
  { value: 'openai', label: 'OpenAI Whisper' }, { value: 'xfyun', label: '讯飞极速听写' },
  { value: 'volcengine', label: '火山引擎语音' }, { value: 'official', label: 'WhisperKey 官方中转' },
]

const languageOptions = [
  { value: 'auto', label: '自动检测' }, { value: 'zh', label: '中文' }, { value: 'en', label: '英文' },
]

onMounted(async () => { await store.load(); hydrateFromConfig() })

function hydrateFromConfig() {
  const cfg = store.config; if (!cfg) return
  llmProvider.value = cfg.llm.provider || 'openai'
  llmBaseUrl.value = cfg.llm.baseUrl || 'https://api.openai.com/v1'
  llmModel.value = cfg.llm.model || 'gpt-4o-mini'
  llmKeyConfigured.value = cfg.llm.apiKeyLen > 0
  llmApiKey.value = llmKeyConfigured.value ? mask(cfg.llm.apiKeyLen) : ''
  llmSecretConfigured.value = cfg.llm.apiSecretLen > 0
  llmSecretKey.value = llmSecretConfigured.value ? mask(cfg.llm.apiSecretLen) : ''
  asrProvider.value = cfg.asr.provider || 'openai'
  asrBaseUrl.value = cfg.asr.baseUrl || 'https://api.openai.com/v1'
  asrModel.value = cfg.asr.model || 'whisper-1'
  asrLanguage.value = cfg.asr.language || 'auto'
  asrKeyConfigured.value = cfg.asr.apiKeyLen > 0
  asrApiKey.value = asrKeyConfigured.value ? mask(cfg.asr.apiKeyLen) : ''
  asrSecretConfigured.value = cfg.asr.apiSecretLen > 0
  asrApiSecret.value = asrSecretConfigured.value ? mask(cfg.asr.apiSecretLen) : ''
}

function isMasked(v: string) { return !!v && v.length > 0 && [...v].every(c => c === '•') }
function keyFocus(r: any) { if (r?.value && isMasked(r.value)) r.value = '' }
function keyBlur(r: any, len: number) { if (r?.value === '' && len > 0) r.value = mask(len) }

async function testLlm() {
  testingLlm.value = true
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const key = (llmApiKey.value && !isMasked(llmApiKey.value)) ? llmApiKey.value : ''
    const sec = (llmSecretKey.value && !isMasked(llmSecretKey.value)) ? llmSecretKey.value : ''
    const result = await invoke<string>('cmd_llm_test_connection', {
      provider: llmProvider.value, apiKey: key, apiSecret: sec || null,
      baseUrl: llmBaseUrl.value, model: llmModel.value,
    })
    message.success(result)
  } catch (e: any) { message.error(typeof e === 'string' ? e : '连接测试失败') }
  finally { testingLlm.value = false }
}

async function testAsr() {
  testingAsr.value = true
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const key = (asrApiKey.value && !isMasked(asrApiKey.value)) ? asrApiKey.value : ''
    const sec = (asrApiSecret.value && !isMasked(asrApiSecret.value)) ? asrApiSecret.value : ''
    const result = await invoke<string>('cmd_asr_test_connection', {
      provider: asrProvider.value, apiKey: key, apiSecret: sec || null,
      baseUrl: asrBaseUrl.value, model: asrModel.value,
    })
    message.success(result)
  } catch (e: any) {
    console.error('[testAsr] caught error:', e, typeof e, JSON.stringify(e))
    message.error(typeof e === 'string' ? e : '连接测试失败')
  }
  finally { testingAsr.value = false }
}
async function saveProviders() {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const llmKey = (llmApiKey.value && !isMasked(llmApiKey.value)) ? llmApiKey.value : ''
    const llmSec = (llmSecretKey.value && !isMasked(llmSecretKey.value)) ? llmSecretKey.value : ''
    await invoke('cmd_llm_set_config', {
      provider: llmProvider.value, apiKey: llmKey, apiSecret: llmSec || null,
      baseUrl: llmBaseUrl.value, model: llmModel.value,
    })
    let asrKey = '', asrSec = ''
    if (asrProvider.value === 'xfyun') {
      asrKey = (asrApiKey.value && !isMasked(asrApiKey.value)) ? asrApiKey.value : ''
      asrSec = (asrApiSecret.value && !isMasked(asrApiSecret.value)) ? asrApiSecret.value : ''
    } else if (asrProvider.value === 'openai' || asrProvider.value === 'volcengine') {
      asrKey = (asrApiKey.value && !isMasked(asrApiKey.value)) ? asrApiKey.value : ''
    }
    await invoke('cmd_asr_set_config', {
      provider: asrProvider.value, apiKey: asrKey, apiSecret: asrSec || null,
      baseUrl: asrBaseUrl.value, model: asrModel.value, language: asrLanguage.value,
    })
    await invoke('cmd_config_save')
    if (store.config) {
      Object.assign(store.config.llm, { provider: llmProvider.value, baseUrl: llmBaseUrl.value, model: llmModel.value })
      Object.assign(store.config.asr, { provider: asrProvider.value, baseUrl: asrBaseUrl.value, model: asrModel.value, language: asrLanguage.value })
      if (llmKey) { store.config.llm.apiKeyLen = llmKey.length; store.config.llm.apiKey = '__ENCRYPTED__'; llmKeyConfigured.value = true; llmApiKey.value = mask(llmKey.length) }
      if (llmSec) { store.config.llm.apiSecretLen = llmSec.length; store.config.llm.apiSecret = '__ENCRYPTED__'; llmSecretConfigured.value = true; llmSecretKey.value = mask(llmSec.length) }
      if (asrKey) { store.config.asr.apiKeyLen = asrKey.length; store.config.asr.apiKey = '__ENCRYPTED__'; asrKeyConfigured.value = true; asrApiKey.value = mask(asrKey.length) }
      if (asrSec) { store.config.asr.apiSecretLen = asrSec.length; store.config.asr.apiSecret = '__ENCRYPTED__'; asrSecretConfigured.value = true; asrApiSecret.value = mask(asrSec.length) }
    }
    dirty.value = false
    message.success('服务商配置已保存')
  } catch (e: any) { message.error(typeof e === 'string' ? e : '保存失败') }
}

// ── Computed placeholders ──
const llmKeyPlaceholder = computed(() => isMasked(llmApiKey.value) ? `${llmApiKey.value} (已配置)` : 'sk-...')
const llmSecretPlaceholder = computed(() => isMasked(llmSecretKey.value) ? `${llmSecretKey.value} (已配置)` : 'Client Secret (SK)')
const asrKeyPlaceholder = computed(() => isMasked(asrApiKey.value) ? `${asrApiKey.value} (已配置)` : 'sk-...')
const asrXfyunPlaceholder = computed(() => isMasked(asrApiKey.value) ? `${asrApiKey.value} (已配置)` : '讯飞应用 ID')
const asrSecretOpenAi = computed(() => isMasked(asrApiSecret.value) ? `${asrApiSecret.value} (已配置)` : '讯飞 API Secret')
</script>

<template>
  <div>
    <!-- ═══════════════ LLM Provider ═══════════════ -->
    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px">
      <h3 style="font-size: 16px; font-weight: 500; margin: 0">全局 LLM Provider</h3>
      <NButton size="small" :loading="testingLlm" @click="testLlm">测试连接</NButton>
    </div>
    <NText depth="3" style="font-size: 12px; margin-bottom: 16px; display: block">
      所有输出模式（优化 / Markdown / 速问 / 自定义）共用此接口
    </NText>

    <NSpace vertical :size="12">
      <div style="display: flex; align-items: center; gap: 12px">
        <NText style="width: 80px">厂商</NText>
        <NSelect v-model:value="llmProvider" :options="llmProviderOptions" style="width: 200px" />
      </div>

      <template v-if="llmProvider !== 'ernie'">
        <div style="display: flex; align-items: center; gap: 12px">
          <NText style="width: 80px">API Key</NText>
          <NInput :value="llmApiKey" type="password" :placeholder="llmKeyPlaceholder" style="width: 320px" @focus="llmKeyFocused = true; keyFocus(llmApiKey)" @blur="llmKeyFocused = false; keyBlur(llmApiKey, store.config?.llm.apiKeyLen || 0)" @update:value="(v: any) => { llmApiKey = v; dirty = true }" />
        </div>
        <div style="display: flex; align-items: center; gap: 12px">
          <NText style="width: 80px">Base URL</NText>
          <NInput v-model:value="llmBaseUrl" style="width: 320px" />
        </div>
        <div style="display: flex; align-items: center; gap: 12px">
          <NText style="width: 80px">Model</NText>
          <NInput v-model:value="llmModel" style="width: 240px" />
        </div>
      </template>
      <template v-else>
        <div style="display: flex; align-items: center; gap: 12px">
          <NText style="width: 80px">API Key</NText>
          <NInput :value="llmApiKey" type="password" :placeholder="llmKeyPlaceholder" style="width: 320px" @focus="llmKeyFocused = true; keyFocus(llmApiKey)" @blur="llmKeyFocused = false; keyBlur(llmApiKey, store.config?.llm.apiKeyLen || 0)" @update:value="(v: any) => { llmApiKey = v; dirty = true }" />
        </div>
        <div style="display: flex; align-items: center; gap: 12px">
          <NText style="width: 80px">Secret Key</NText>
          <NInput :value="llmSecretKey" type="password" :placeholder="llmSecretPlaceholder" style="width: 320px" @focus="llmSecretFocused = true; keyFocus(llmSecretKey)" @blur="llmSecretFocused = false; keyBlur(llmSecretKey, store.config?.llm.apiSecretLen || 0)" @update:value="(v: any) => (llmSecretKey = v)" />
        </div>
        <div style="display: flex; align-items: center; gap: 12px">
          <NText style="width: 80px">Model</NText>
          <NInput v-model:value="llmModel" placeholder="ernie-4.0-turbo-8k" style="width: 240px" />
        </div>
      </template>
    </NSpace>

    <NDivider />

    <!-- ═══════════════ ASR Provider ═══════════════ -->
    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px">
      <h3 style="font-size: 16px; font-weight: 500; margin: 0">ASR 语音识别</h3>
      <NButton size="small" :loading="testingAsr" @click="testAsr">测试连接</NButton>
    </div>

    <NSpace vertical :size="12">
      <div style="display: flex; align-items: center; gap: 12px">
        <NText style="width: 80px">厂商</NText>
        <NSelect v-model:value="asrProvider" :options="asrProviderOptions" style="width: 220px" />
      </div>

      <template v-if="asrProvider === 'openai'">
        <div style="display: flex; align-items: center; gap: 12px">
          <NText style="width: 80px">API Key</NText>
          <NInput :value="asrApiKey" type="password" :placeholder="asrKeyPlaceholder" style="width: 320px" @focus="asrKeyFocused = true; keyFocus(asrApiKey)" @blur="asrKeyFocused = false; keyBlur(asrApiKey, store.config?.asr.apiKeyLen || 0)" @update:value="(v: any) => { asrApiKey = v; dirty = true }" />
        </div>
        <div style="display: flex; align-items: center; gap: 12px"><NText style="width: 80px">Base URL</NText><NInput v-model:value="asrBaseUrl" placeholder="https://api.openai.com/v1" style="width: 320px" /></div>
        <div style="display: flex; align-items: center; gap: 12px"><NText style="width: 80px">Model</NText><NInput v-model:value="asrModel" placeholder="whisper-1" style="width: 200px" /></div>
        <div style="display: flex; align-items: center; gap: 12px"><NText style="width: 80px">语言</NText><NSelect v-model:value="asrLanguage" :options="languageOptions" style="width: 140px" /></div>
      </template>

      <template v-else-if="asrProvider === 'xfyun'">
        <div style="display: flex; align-items: center; gap: 12px">
          <NText style="width: 80px">AppId</NText>
          <NInput :value="asrApiKey" :placeholder="asrXfyunPlaceholder" style="width: 320px" @focus="keyFocus(asrApiKey)" @blur="keyBlur(asrApiKey, store.config?.asr.apiKeyLen || 0)" @update:value="(v: any) => { asrApiKey = v; dirty = true }" />
        </div>
        <div style="display: flex; align-items: center; gap: 12px">
          <NText style="width: 80px">API Secret</NText>
          <NInput :value="asrApiSecret" type="password" :placeholder="asrSecretOpenAi" style="width: 320px" @focus="asrSecretFocused = true; keyFocus(asrApiSecret)" @blur="asrSecretFocused = false; keyBlur(asrApiSecret, store.config?.asr.apiSecretLen || 0)" @update:value="(v: any) => { asrApiSecret = v; dirty = true }" />
        </div>
        <div style="display: flex; align-items: center; gap: 12px"><NText style="width: 80px">接口地址</NText><NInput v-model:value="asrBaseUrl" placeholder="https://raasr.xfyun.cn" style="width: 320px" /></div>
        <div style="display: flex; align-items: center; gap: 12px"><NText style="width: 80px">语言</NText><NSelect v-model:value="asrLanguage" :options="languageOptions" style="width: 140px" /></div>
      </template>

      <template v-else-if="asrProvider === 'volcengine'">
        <div style="display: flex; align-items: center; gap: 12px">
          <NText style="width: 80px">APP Key</NText>
          <NInput :value="asrApiKey" placeholder="新版控制台的 APP Key" style="width: 320px" @focus="keyFocus(asrApiKey)" @blur="keyBlur(asrApiKey, store.config?.asr.apiKeyLen || 0)" @update:value="(v: any) => { asrApiKey = v; dirty = true }" />
        </div>
        <NText depth="3" style="font-size: 11px; margin-top: -8px; display: block">
          新版控制台只需 X-Api-Key，前往 豆包语音控制台 → 应用管理 → 应用详情 获取
        </NText>
        <div style="display: flex; align-items: center; gap: 12px"><NText style="width: 80px">接口地址</NText><NInput v-model:value="asrBaseUrl" placeholder="https://openspeech.bytedance.com" style="width: 320px" /></div>
      </template>

      <template v-else-if="asrProvider === 'official'">
        <NText depth="2" style="font-size: 13px; padding: 12px 0">使用 WhisperKey 官方中转服务，无需额外配置。免费额度内直接使用。</NText>
      </template>
    </NSpace>

    <NDivider />
    <NButton type="primary" @click="saveProviders">保存配置</NButton>
  </div>
</template>

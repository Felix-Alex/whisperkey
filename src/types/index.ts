export interface Config {
  version: number
  hotkey: HotkeyConfig
  modes: ModesConfig
  asr: AsrConfig
  providers: ProvidersConfig
  audio: AudioConfig
  ui: UiConfig
  system: SystemConfig
  history: HistoryConfig
  advanced: AdvancedConfig
}

export interface HotkeyConfig {
  modifiers: string[]
  key: string
  paused: boolean
}

export interface ModesConfig {
  default: string
  raw: ModeAssignment
  polish: ModeAssignment
  markdown: ModeAssignment
}

export interface ModeAssignment {
  llmProvider: string
  llmModel: string
}

export interface AsrConfig {
  default: string
  language: string
}

export interface ProvidersConfig {
  openai: ProviderCredential
  anthropic: ProviderCredential
  deepseek: ProviderCredential
  qwen: ProviderCredential
  ernie: ProviderCredential
  doubao: ProviderCredential
  gemini: ProviderCredential
  xfyun: ProviderCredential
  volcengine: ProviderCredential
  official: Record<string, never>
}

export interface ProviderCredential {
  apiKey?: string
  baseUrl?: string
}

export interface AudioConfig {
  maxDurationSec: number
  silenceAutoStop: boolean
  silenceTimeoutMs: number
  inputDevice: string
}

export interface UiConfig {
  theme: string
  language: string
  indicatorPosition: string
}

export interface SystemConfig {
  autoStart: boolean
  minimizeToTray: boolean
  checkUpdates: boolean
}

export interface HistoryConfig {
  enabled: boolean
  retentionDays: number
}

export interface AdvancedConfig {
  logLevel: string
  telemetry: boolean
}

export interface RecordingState {
  isRecording: boolean
  elapsedMs: number
  level: number
  mode: string
}

export interface HistoryItem {
  id: number
  createdAt: number
  mode: string
  rawText: string
  processedText: string
  durationMs: number
  appName?: string
  injected: boolean
}

export interface LicenseStatus {
  activated: boolean
  products: string[]
  expiresAt?: number
}

export type IndicatorState = 'Idle' | 'Recording' | 'Processing'

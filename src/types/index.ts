export type OutputMode = 'raw' | 'polish' | 'markdown' | 'quick_ask' | 'custom'

export interface Config {
  version: number
  hotkey: HotkeyConfig
  llm: LlmConfig
  asr: AsrConfig
  audio: AudioConfig
  ui: UiConfig
  system: SystemConfig
  history: HistoryConfig
  advanced: AdvancedConfig
  outputMode: OutputMode
}

export interface HotkeyConfig {
  modifiers: string[]
  key: string
  paused: boolean
}

export interface LlmConfig {
  provider: string
  apiKey: string
  apiKeyLen: number
  apiSecret: string
  apiSecretLen: number
  baseUrl: string
  model: string
}

export interface AsrConfig {
  provider: string
  apiKey: string
  apiKeyLen: number
  apiSecret: string
  apiSecretLen: number
  baseUrl: string
  model: string
  language: string
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
  mode: OutputMode
  rawText: string
  processedText: string
  durationMs: number
  appName?: string
  asrProvider?: string
  llmProvider?: string
  injected: boolean
}

export interface LicenseStatus {
  activated: boolean
  expiresAt?: number
}

export type IndicatorState = 'Idle' | 'Recording' | 'Processing'

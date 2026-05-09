import { invoke } from '@tauri-apps/api/core'
import type { Config } from '../types'

export async function getConfig(): Promise<Config> {
  return invoke<Config>('cmd_config_get')
}

export async function setConfig(config: Config): Promise<void> {
  return invoke('cmd_config_set', { config })
}

export async function setProviderKey(provider: string, key: string): Promise<void> {
  return invoke('cmd_config_set_provider_key', { provider, key })
}

export async function testProviderConnection(provider: string): Promise<boolean> {
  return invoke<boolean>('cmd_provider_test_connection', { provider })
}

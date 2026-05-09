import { invoke } from '@tauri-apps/api/core'

export interface ProviderInfo {
  name: string
  configured: boolean
  testOk: boolean
}

export async function listProviders(): Promise<ProviderInfo[]> {
  return invoke<ProviderInfo[]>('cmd_providers_list')
}

export async function testProvider(provider: string): Promise<boolean> {
  return invoke<boolean>('cmd_providers_test', { provider })
}

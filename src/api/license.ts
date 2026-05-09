import { invoke } from '@tauri-apps/api/core'
import type { LicenseStatus } from '../types'

export async function getLicenseStatus(): Promise<LicenseStatus> {
  return invoke<LicenseStatus>('cmd_license_status')
}

export async function activateLicense(code: string): Promise<LicenseStatus> {
  return invoke<LicenseStatus>('cmd_license_activate', { code })
}

export async function unbindLicense(): Promise<void> {
  return invoke('cmd_license_unbind')
}

export async function quitApp(): Promise<void> {
  return invoke('cmd_app_quit')
}

export async function openLogsFolder(): Promise<void> {
  return invoke('cmd_app_open_logs_folder')
}

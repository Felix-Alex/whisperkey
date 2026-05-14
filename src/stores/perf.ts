import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface PerfRecord {
  ts: number
  mode: string
  durationMs: number
  audioLen: number
  provider: string
  success: boolean
}

export const usePerfStore = defineStore('perf', () => {
  const records = ref<PerfRecord[]>([])
  const expanded = ref(false)

  function addRecord(rec: PerfRecord) {
    records.value.unshift(rec)
    if (records.value.length > 20) records.value.pop()
  }

  function toggle() {
    expanded.value = !expanded.value
  }

  function clear() {
    records.value = []
  }

  return { records, expanded, addRecord, toggle, clear }
})

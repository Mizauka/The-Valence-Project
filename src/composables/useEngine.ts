// ─── useEngine ────────────────────────────────────────────────────
// Engine initialization state for App.vue.
// Provides reactive loading/error states for the WASM engine.

import { ref, readonly } from 'vue'
import { waitForEngine } from '../services/engineStore'

const isReady = ref(false)
const isLoading = ref(false)
const error = ref<string | null>(null)
let _initStarted = false

export function useEngine() {
  async function init(): Promise<void> {
    if (_initStarted) return
    _initStarted = true
    isLoading.value = true
    error.value = null
    try {
      await waitForEngine()
      isReady.value = true
    } catch (e: any) {
      error.value = e?.message || 'Failed to initialize engine'
      console.error('[useEngine] init failed:', e)
    } finally {
      isLoading.value = false
    }
  }

  return {
    isReady: readonly(isReady),
    isLoading: readonly(isLoading),
    error: readonly(error),
    init,
  }
}

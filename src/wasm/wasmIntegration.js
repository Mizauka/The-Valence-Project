let engine = null
let initPromise = null
let loadError = null

export async function getEngine() {
  if (engine) return engine
  if (loadError) throw loadError
  if (!initPromise) {
    initPromise = _init().catch((err) => {
      loadError = err
      console.error('[WASM] failed to initialize:', err)
    })
  }
  await initPromise
  return engine
}

async function _init() {
  console.log('[WASM] initializing ValenceEngine (web target)...')

  const jsUrl = new URL('wasm/wasm_core.js', document.baseURI).href
  const wasmUrl = new URL('wasm/wasm_core_bg.wasm', document.baseURI).href

  const response = await fetch(jsUrl)
  if (!response.ok) {
    throw new Error(`Failed to fetch WASM JS glue: HTTP ${response.status}`)
  }

  const jsCode = await response.text()
  console.log('[WASM] fetched JS glue, size:', jsCode.length, 'bytes')

  const blob = new Blob([jsCode], { type: 'application/javascript' })
  const blobUrl = URL.createObjectURL(blob)

  try {
    const mod = await import(/* @vite-ignore */ blobUrl)

    if (mod.default && typeof mod.default === 'function') {
      await mod.default({ module_or_path: wasmUrl })
    }

    const { ValenceEngine } = mod

    if (!ValenceEngine) {
      throw new Error('ValenceEngine not found in WASM module exports')
    }

    engine = new ValenceEngine()

    if (!engine || typeof engine.runSimulation !== 'function') {
      throw new Error('ValenceEngine instance invalid')
    }

    console.log('[WASM] ValenceEngine loaded successfully ✓')
    return engine
  } finally {
    URL.revokeObjectURL(blobUrl)
  }
}

export function isLoaded() {
  return engine !== null
}

export function getLoadError() {
  return loadError
}

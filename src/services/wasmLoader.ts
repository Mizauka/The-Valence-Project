// ─── WASM Module Loader ───────────────────────────────────────────
// Loads the wasm-pack generated JS glue and instantiates ValenceEngine.
// Uses direct import() from server URL so that relative imports
// (./wasm_core_bg.js) resolve correctly against the server path.
// IMPORTANT: files are in public/wasm/, served at /wasm/ from Vite root.

let engine: any = null
let initPromise: Promise<any> | null = null
let loadError: Error | null = null

/**
 * Get the raw server URL for a public asset.
 * Uses Vite's BASE_URL: '/' in dev, '/reValenceGUI/' in prod.
 */
function getPublicAssetUrl(filename: string): string {
  const base = (import.meta as any).env?.BASE_URL
  if (!base || base === '/') {
    return window.location.origin + '/' + filename
  }
  return window.location.origin + base + filename
}

async function initWasm(): Promise<any> {
  console.log('[WASM] initializing ValenceEngine (web target)...')

  const jsUrl = getPublicAssetUrl('wasm/wasm_core.js')
  const wasmUrl = getPublicAssetUrl('wasm/wasm_core_bg.wasm')

  console.log('[WASM] loading from:', jsUrl)

  // Direct dynamic import preserves the base URL for relative module resolution
  const mod = await import(/* @vite-ignore */ jsUrl)

  console.log('[WASM] module loaded, initializing WASM...')

  // wasm-pack target=web: default export is the init function
  if (mod.default && typeof mod.default === 'function') {
    await mod.default({ module_or_path: wasmUrl })
  }

  const { ValenceEngine } = mod
  if (!ValenceEngine) {
    throw new Error('ValenceEngine not found in WASM module exports')
  }

  const instance = new ValenceEngine()
  if (!instance || typeof instance.runSimulation !== 'function') {
    throw new Error('ValenceEngine instance invalid')
  }

  console.log('[WASM] ValenceEngine loaded successfully ✓')
  return instance
}

export async function getEngine(): Promise<any> {
  if (engine) return engine
  if (loadError) throw loadError
  if (!initPromise) {
    initPromise = initWasm().catch((err: Error) => {
      loadError = err
      console.error('[WASM] failed to initialize:', err)
      throw err
    })
  }
  await initPromise
  engine = await initPromise
  return engine
}

export function isLoaded(): boolean {
  return engine !== null
}

export function getLoadError(): Error | null {
  return loadError
}

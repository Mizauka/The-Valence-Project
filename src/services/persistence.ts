// ─── Persistence Layer ────────────────────────────────────────────
// OPFS (Origin Private File System) + IndexedDB for directory handles.
// Falls back to localStorage when OPFS is unavailable.

const IDB_NAME = 'valence_handles'
const IDB_STORE = 'handles'
const IDB_KEY = 'externalDir'

function openIDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(IDB_NAME, 1)
    req.onupgradeneeded = () => {
      req.result.createObjectStore(IDB_STORE)
    }
    req.onsuccess = () => resolve(req.result)
    req.onerror = () => reject(req.error)
  })
}

async function idbPut(key: string, value: any): Promise<void> {
  const db = await openIDB()
  return new Promise((resolve, reject) => {
    const tx = db.transaction(IDB_STORE, 'readwrite')
    tx.objectStore(IDB_STORE).put(value, key)
    tx.oncomplete = () => resolve()
    tx.onerror = () => reject(tx.error)
  })
}

async function idbGet(key: string): Promise<any> {
  const db = await openIDB()
  return new Promise((resolve, reject) => {
    const tx = db.transaction(IDB_STORE, 'readonly')
    const req = tx.objectStore(IDB_STORE).get(key)
    req.onsuccess = () => resolve(req.result)
    req.onerror = () => reject(req.error)
  })
}

// ─── OPFS ─────────────────────────────────────────────────────────
let _opfsRoot: FileSystemDirectoryHandle | null = null

async function getOPFS(): Promise<FileSystemDirectoryHandle | null> {
  if (_opfsRoot) return _opfsRoot
  if (typeof navigator === 'undefined' || !navigator.storage) return null
  try {
    const root = await navigator.storage.getDirectory()
    _opfsRoot = await root.getDirectoryHandle('valence', { create: true })
    console.log('[persistence] OPFS ready at /valence/')
    return _opfsRoot
  } catch (e) {
    console.error('[persistence] OPFS init failed:', e)
    return null
  }
}

export function writeFallback(key: string, value: string): void {
  try { localStorage.setItem('valence_' + key, value) } catch { /* ignore */ }
}

export function readFallback(key: string): string | null {
  try { return localStorage.getItem('valence_' + key) } catch { return null }
}

export async function opfsWrite(fileName: string, content: string): Promise<void> {
  const dir = await getOPFS()
  if (!dir) { writeFallback(fileName, content); return }
  try {
    const handle = await dir.getFileHandle(fileName, { create: true })
    const writable = await handle.createWritable()
    await writable.write(content)
    await writable.close()
  } catch (e) {
    console.error(`[persistence] OPFS write ${fileName} failed:`, e)
    writeFallback(fileName, content)
  }
}

export async function opfsRead(fileName: string): Promise<string | null> {
  const dir = await getOPFS()
  if (!dir) return readFallback(fileName)
  try {
    const handle = await dir.getFileHandle(fileName)
    const file = await handle.getFile()
    return await file.text()
  } catch {
    return readFallback(fileName)
  }
}

// ─── External Directory (File System Access API) ──────────────────
let _externalDirHandle: FileSystemDirectoryHandle | null = null

export async function restoreExternalDir(): Promise<void> {
  try {
    const handle = await idbGet(IDB_KEY)
    if (!handle) return

    const perm = await (handle as any).queryPermission({ mode: 'readwrite' })
    if (perm === 'granted') {
      _externalDirHandle = handle
      console.log('[persistence] restored external dir:', handle.name)
    }
  } catch (e) {
    console.warn('[persistence] failed to restore external dir handle:', e)
  }
}

export function getExternalDirHandle(): FileSystemDirectoryHandle | null {
  return _externalDirHandle
}

export async function getExternalDirName(): Promise<string | null> {
  if (_externalDirHandle) return _externalDirHandle.name
  try {
    const handle = await idbGet(IDB_KEY)
    if (handle) return handle.name
  } catch { /* ignore */ }
  return null
}

export async function checkExternalDirPermission(): Promise<boolean> {
  try {
    const handle = _externalDirHandle || await idbGet(IDB_KEY)
    if (!handle) return false
    const perm = await (handle as any).queryPermission({ mode: 'readwrite' })
    return perm === 'granted'
  } catch {
    return false
  }
}

export async function requestExternalDirPermission(): Promise<boolean> {
  try {
    const handle = await idbGet(IDB_KEY)
    if (!handle) return false
    const perm = await (handle as any).requestPermission({ mode: 'readwrite' })
    if (perm === 'granted') {
      _externalDirHandle = handle
      return true
    }
    return false
  } catch {
    return false
  }
}

export async function pickDataDirectory(): Promise<string> {
  if (!('showDirectoryPicker' in window)) {
    throw new Error('File System Access API not supported')
  }

  _externalDirHandle = await (window as any).showDirectoryPicker({ mode: 'readwrite' })
  await idbPut(IDB_KEY, _externalDirHandle)
  console.log('[persistence] picked external dir:', _externalDirHandle!.name)
  return _externalDirHandle!.name
}

async function readExternalFile(fileName: string): Promise<string | null> {
  if (!_externalDirHandle) return null
  try {
    const handle = _externalDirHandle as any
    const perm = await handle.queryPermission({ mode: 'readwrite' })
    if (perm !== 'granted') return null

    let dirHandle = _externalDirHandle
    try {
      dirHandle = await _externalDirHandle.getDirectoryHandle('data')
    } catch { /* no data subdir, use root */ }

    const fileHandle = await dirHandle.getFileHandle(fileName)
    const file = await fileHandle.getFile()
    return await file.text()
  } catch {
    return null
  }
}

export async function writeExternalFile(fileName: string, content: string): Promise<void> {
  if (!_externalDirHandle) return
  try {
    const handle = _externalDirHandle as any
    const perm = await handle.queryPermission({ mode: 'readwrite' })
    if (perm !== 'granted') return

    let dataDir = _externalDirHandle
    try {
      dataDir = await _externalDirHandle.getDirectoryHandle('data', { create: true })
    } catch { /* use root */ }

    const fh = await dataDir.getFileHandle(fileName, { create: true })
    const w = await fh.createWritable()
    await w.write(content)
    await w.close()
  } catch (e) {
    console.warn('[persistence] external write failed:', e)
  }
}

export { readExternalFile }

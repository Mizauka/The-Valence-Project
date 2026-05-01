let _engine = null
let _initPromise = null
let _opfsRoot = null
let _externalDirHandle = null
let _drugMetaCache = {}

const IDB_NAME = 'valence_handles'
const IDB_STORE = 'handles'
const IDB_KEY = 'externalDir'

function openIDB() {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(IDB_NAME, 1)
    req.onupgradeneeded = () => {
      req.result.createObjectStore(IDB_STORE)
    }
    req.onsuccess = () => resolve(req.result)
    req.onerror = () => reject(req.error)
  })
}

async function idbPut(key, value) {
  const db = await openIDB()
  return new Promise((resolve, reject) => {
    const tx = db.transaction(IDB_STORE, 'readwrite')
    tx.objectStore(IDB_STORE).put(value, key)
    tx.oncomplete = () => resolve()
    tx.onerror = () => reject(tx.error)
  })
}

async function idbGet(key) {
  const db = await openIDB()
  return new Promise((resolve, reject) => {
    const tx = db.transaction(IDB_STORE, 'readonly')
    const req = tx.objectStore(IDB_STORE).get(key)
    req.onsuccess = () => resolve(req.result)
    req.onerror = () => reject(req.error)
  })
}

export async function getEngine() {
  if (_engine) return _engine
  if (!_initPromise) {
    _initPromise = _init()
  }
  await _initPromise
  return _engine
}

async function getOPFS() {
  if (_opfsRoot) return _opfsRoot
  if (typeof navigator === 'undefined' || !navigator.storage) return null
  try {
    const root = await navigator.storage.getDirectory()
    _opfsRoot = await root.getDirectoryHandle('valence', { create: true })
    console.log('[engineStore] OPFS ready at /valence/')
    return _opfsRoot
  } catch (e) {
    console.error('[engineStore] OPFS init failed:', e)
    return null
  }
}

async function opfsWrite(fileName, content) {
  const dir = await getOPFS()
  if (!dir) { writeFallback(fileName, content); return }
  try {
    const handle = await dir.getFileHandle(fileName, { create: true })
    const writable = await handle.createWritable()
    await writable.write(content)
    await writable.close()
  } catch (e) {
    console.error(`[engineStore] OPFS write ${fileName} failed:`, e)
    writeFallback(fileName, content)
  }
}

async function opfsRead(fileName) {
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

function writeFallback(key, value) {
  try { localStorage.setItem('valence_' + key, value) } catch {}
}

function readFallback(key) {
  try { return localStorage.getItem('valence_' + key) } catch { return null }
}

async function _init() {
  const cacheBust = `?v=${Date.now()}`
  const jsUrl = '/wasm/wasm_core.js' + cacheBust
  const wasmUrl = '/wasm/wasm_core_bg.wasm' + cacheBust

  const response = await fetch(jsUrl)
  if (!response.ok) throw new Error(`Failed to fetch WASM JS glue: HTTP ${response.status}`)

  const jsCode = await response.text()
  const blob = new Blob([jsCode], { type: 'application/javascript' })
  const blobUrl = URL.createObjectURL(blob)

  try {
    const mod = await import(/* @vite-ignore */ blobUrl)

    if (mod.default && typeof mod.default === 'function') {
      await mod.default({ module_or_path: wasmUrl })
    }

    const { ValenceEngine } = mod
    if (!ValenceEngine) throw new Error('ValenceEngine not found in WASM module exports')

    _engine = new ValenceEngine()
    console.log('[engineStore] ValenceEngine initialized')

    await restoreExternalDir()

    const savedWeightRaw = await opfsRead('weight')
    if (savedWeightRaw) {
      const w = parseFloat(savedWeightRaw)
      if (!isNaN(w) && w > 0) _engine.setWeight(w)
    }

    await loadPresetDrugs(_engine)

    const externalLoaded = _externalDirHandle
      ? await loadFromExternalDir(_engine)
      : false

    if (!externalLoaded) {
      await loadCustomDrugsFromOPFS(_engine)
      await loadDosesFromOPFS(_engine)
    }

    if (_externalDirHandle) {
      await saveAll(_engine)
    }
  } finally {
    URL.revokeObjectURL(blobUrl)
  }
}

async function restoreExternalDir() {
  try {
    const handle = await idbGet(IDB_KEY)
    if (!handle) return

    const perm = await handle.queryPermission({ mode: 'readwrite' })
    if (perm === 'granted') {
      _externalDirHandle = handle
      console.log('[engineStore] restored external dir:', handle.name)
    } else {
      console.log('[engineStore] external dir permission not granted, will re-ask on sync')
    }
  } catch (e) {
    console.warn('[engineStore] failed to restore external dir handle:', e)
  }
}

export async function getExternalDirName() {
  if (_externalDirHandle) return _externalDirHandle.name
  try {
    const handle = await idbGet(IDB_KEY)
    if (handle) return handle.name
  } catch {}
  return null
}

export async function checkExternalDirPermission() {
  try {
    const handle = _externalDirHandle || await idbGet(IDB_KEY)
    if (!handle) return false
    const perm = await handle.queryPermission({ mode: 'readwrite' })
    return perm === 'granted'
  } catch {
    return false
  }
}

export async function requestExternalDirPermission() {
  try {
    const handle = await idbGet(IDB_KEY)
    if (!handle) return false

    const perm = await handle.requestPermission({ mode: 'readwrite' })
    if (perm === 'granted') {
      _externalDirHandle = handle
      return true
    }
    return false
  } catch {
    return false
  }
}

async function readExternalFile(fileName) {
  if (!_externalDirHandle) return null
  try {
    const perm = await _externalDirHandle.queryPermission({ mode: 'readwrite' })
    if (perm !== 'granted') return null

    let dirHandle = _externalDirHandle
    try {
      dirHandle = await _externalDirHandle.getDirectoryHandle('data')
    } catch {
      // no data subdirectory, try root
    }

    const fileHandle = await dirHandle.getFileHandle(fileName)
    const file = await fileHandle.getFile()
    return await file.text()
  } catch {
    return null
  }
}

async function loadFromExternalDir(engine) {
  const dosesRaw = await readExternalFile('doses.json')
  const drugsRaw = await readExternalFile('custom_drugs.json')

  if (!dosesRaw && !drugsRaw) return false

  if (drugsRaw) {
    try {
      const drugs = JSON.parse(drugsRaw)
      if (Array.isArray(drugs)) {
        for (const d of drugs) {
          engine.registerDrug(jsonDrugToRecord(d))
        }
        console.log(`[engineStore] loaded ${drugs.length} custom drugs from external dir`)
      }
    } catch (e) {
      console.warn('[engineStore] failed to parse external custom_drugs:', e)
    }
  }

  if (dosesRaw) {
    try {
      const data = JSON.parse(dosesRaw)

      if (data.weight && !isNaN(data.weight) && data.weight > 0) {
        engine.setWeight(data.weight)
      }

      if (Array.isArray(data.events)) {
        for (const ev of data.events) {
          const drugId = resolveDrugId(engine, ev)
          if (!drugId) continue
          engine.addDose({
            dose_id: ev.id || crypto.randomUUID(),
            drug_id: drugId,
            dose_amount: ev.doseMG || 0,
            timestamp: ev.timeH || 0,
            route: ev.route || 'oral',
          })
        }
        console.log(`[engineStore] loaded ${data.events.length} doses from external dir`)
      }
    } catch (e) {
      console.warn('[engineStore] failed to parse external doses:', e)
    }
  }

  return true
}

function resolveDrugId(engine, ev) {
  if (ev.extras?.drug_id) return ev.extras.drug_id

  const ester = ev.ester
  if (!ester) return null

  const allDrugs = JSON.parse(JSON.stringify(engine.getAllDrugs()))
  const byId = allDrugs.find(d => d.drug_id === ester)
  if (byId) return ester

  const byName = allDrugs.find(d => d.name === ester)
  if (byName) return byName.drug_id

  console.warn(`[engineStore] cannot resolve drug: "${ester}", skipping`)
  return null
}

async function loadPresetDrugs(engine) {
  try {
    const [hrtResp, journalResp] = await Promise.all([
      fetch('/data/hrt_drugs.json'),
      fetch('/data/journal_drugs.json'),
    ])
    const hrt = await hrtResp.json()
    const journal = await journalResp.json()

    const all = [...hrt, ...journal].map(d => {
      _drugMetaCache[d.drug_id] = {
        dose_unit: d.dose_unit || 'mg',
        routes: d.routes || [{ route: 'oral', unit: d.dose_unit || 'mg' }],
      }
      return jsonDrugToRecord(d)
    })
    engine.registerDrugs(all)
    console.log(`[engineStore] loaded ${all.length} preset drugs`)
  } catch (e) {
    console.error('[engineStore] failed to load preset drugs:', e)
  }
}

async function loadCustomDrugsFromOPFS(engine) {
  const raw = await opfsRead('custom_drugs.json')
  if (!raw) return

  try {
    const drugs = JSON.parse(raw)
    if (Array.isArray(drugs)) {
      for (const d of drugs) {
        _drugMetaCache[d.drug_id] = {
          dose_unit: d.dose_unit || 'mg',
          routes: d.routes || [{ route: 'oral', unit: d.dose_unit || 'mg' }],
        }
        engine.registerDrug(jsonDrugToRecord(d))
      }
      console.log(`[engineStore] loaded ${drugs.length} custom drugs from OPFS`)
    }
  } catch (e) {
    console.warn('[engineStore] failed to parse custom_drugs:', e)
  }
}

async function loadDosesFromOPFS(engine) {
  const raw = await opfsRead('doses.json')
  if (!raw) return

  let data
  try { data = JSON.parse(raw) } catch { return }

  if (data.weight && !isNaN(data.weight) && data.weight > 0) {
    engine.setWeight(data.weight)
  }

  if (!Array.isArray(data.events)) return

  for (const ev of data.events) {
    const drugId = resolveDrugId(engine, ev)
    if (!drugId) continue
    engine.addDose({
      dose_id: ev.id || crypto.randomUUID(),
      drug_id: drugId,
      dose_amount: ev.doseMG || 0,
      timestamp: ev.timeH || 0,
      route: ev.route || 'oral',
    })
  }

  console.log(`[engineStore] loaded ${data.events.length} doses from OPFS`)
}

async function saveAll(engine) {
  const rawDrugs = engine.getAllDrugs()
  const allDrugs = JSON.parse(JSON.stringify(rawDrugs))
  const customDrugs = allDrugs.filter(d =>
    !d.drug_id.startsWith('hrt_') && !d.drug_id.startsWith('journal_')
  )

  const rawDoses = JSON.parse(JSON.stringify(engine.getAllDoses()))
  const rawDrugList = JSON.parse(JSON.stringify(engine.getAllDrugs()))
  const drugMap = {}
  for (const d of rawDrugList) { drugMap[d.drug_id] = d.name }

  const events = rawDoses.map(d => ({
    id: d.dose_id,
    route: d.route || 'oral',
    ester: drugMap[d.drug_id] || d.drug_id,
    timeH: d.timestamp,
    doseMG: d.dose_amount,
    extras: { drug_id: d.drug_id },
  }))

  const payload = {
    meta: { version: 1, exportedAt: new Date().toISOString() },
    weight: engine.getWeight(),
    events,
  }

  await Promise.all([
    opfsWrite('custom_drugs.json', JSON.stringify(customDrugs, null, 2)),
    opfsWrite('doses.json', JSON.stringify(payload, null, 2)),
    opfsWrite('weight', String(engine.getWeight())),
  ])

  if (_externalDirHandle) {
    await syncToExternalDir(engine, customDrugs, payload)
  }
}

async function syncToExternalDir(engine, customDrugs, payload) {
  if (!_externalDirHandle) return
  try {
    const perm = await _externalDirHandle.queryPermission({ mode: 'readwrite' })
    if (perm !== 'granted') return

    let dataDir = _externalDirHandle
    try {
      dataDir = await _externalDirHandle.getDirectoryHandle('data', { create: true })
    } catch {
      // use root
    }

    const cdFile = await dataDir.getFileHandle('custom_drugs.json', { create: true })
    const cdW = await cdFile.createWritable()
    await cdW.write(JSON.stringify(customDrugs, null, 2))
    await cdW.close()

    const dsFile = await dataDir.getFileHandle('doses.json', { create: true })
    const dsW = await dsFile.createWritable()
    await dsW.write(JSON.stringify(payload, null, 2))
    await dsW.close()

    console.log('[engineStore] synced to', _externalDirHandle.name)
  } catch (e) {
    console.warn('[engineStore] sync failed:', e)
  }
}

export async function pickDataDirectory() {
  if (!('showDirectoryPicker' in window)) throw new Error('File System Access API not supported')

  _externalDirHandle = await window.showDirectoryPicker({ mode: 'readwrite' })
  await idbPut(IDB_KEY, _externalDirHandle)

  const engine = await getEngine()

  await loadFromExternalDir(engine)

  await saveAll(engine)

  console.log('[engineStore] synced to', _externalDirHandle.name)
  return _externalDirHandle.name
}

export function jsonDrugToRecord(d) {
  const p = d.parameters || {}
  const routes = (d.routes || [{ route: 'oral', unit: d.dose_unit || 'mg' }]).map(r => ({
    route: r.route || 'oral',
    unit: r.unit || d.dose_unit || 'mg',
  }))
  return {
    drug_id: d.drug_id || '',
    name: d.name || '',
    model_type: d.model_type || 'one_compartment',
    group_id: d.group_id || '',
    dose_unit: d.dose_unit || 'mg',
    routes,
    display_unit: d.display_unit || '',
    molecular_weight: d.molecular_weight || 0,
    depot_model: d.depot_model || false,
    parameters: { ...p },
  }
}

export async function getAllDrugsWithSource() {
  const engine = await getEngine()
  const raw = engine.getAllDrugs()
  const all = JSON.parse(JSON.stringify(raw))

  for (const d of all) {
    if (d.drug_id.startsWith('hrt_')) d.source = 'hrt'
    else if (d.drug_id.startsWith('journal_')) d.source = 'journal'
    else d.source = 'custom'
    const meta = _drugMetaCache[d.drug_id]
    d.dose_unit = meta?.dose_unit || 'mg'
    d.routes = meta?.routes || [{ route: 'oral', unit: d.dose_unit }]
    d.parameters = d.params || {}
    d.parameters.group_id = d.group_id || ''
  }

  return all
}

export async function getCustomDrugs() {
  const all = await getAllDrugsWithSource()
  return all.filter(d => d.source === 'custom')
}

export async function getPresetDrugs(source) {
  const all = await getAllDrugsWithSource()
  return source ? all.filter(d => d.source === source) : all.filter(d => d.source !== 'custom')
}

export async function addDrug(drugData) {
  const engine = await getEngine()
  _drugMetaCache[drugData.drug_id] = {
    dose_unit: drugData.dose_unit || 'mg',
    routes: drugData.routes || [{ route: 'oral', unit: drugData.dose_unit || 'mg' }],
  }
  engine.registerDrug(jsonDrugToRecord(drugData))
  await saveAll(engine)
}

export async function deleteDrug(drugId) {
  const engine = await getEngine()
  engine.removeDrug(drugId)
  await saveAll(engine)
}

export async function addDose(doseData) {
  const engine = await getEngine()
  engine.addDose({
    dose_id: doseData.dose_id,
    drug_id: doseData.drug_id,
    dose_amount: doseData.dose_amount || 0,
    timestamp: doseData.timestamp,
    route: doseData.route_of_administration || doseData.route || 'oral',
  })
  await saveAll(engine)
}

export async function removeDose(doseId) {
  const engine = await getEngine()
  engine.removeDose(doseId)
  await saveAll(engine)
}

export async function getAllDoses() {
  const engine = await getEngine()
  const raw = engine.getAllDoses()
  const doses = JSON.parse(JSON.stringify(raw))

  const rawDrugs = engine.getAllDrugs()
  const drugMap = {}
  for (const d of JSON.parse(JSON.stringify(rawDrugs))) {
    drugMap[d.drug_id] = d.name
  }

  return doses.map(d => {
    const meta = _drugMetaCache[d.drug_id]
    const doseUnit = meta?.dose_unit || 'mg'
    let displayAmount = d.dose_amount
    let displayUnit = 'mg'
    if (doseUnit === 'µg') { displayAmount = d.dose_amount * 1000; displayUnit = 'µg' }
    else if (doseUnit === 'ng') { displayAmount = d.dose_amount * 1000000; displayUnit = 'ng' }
    else if (doseUnit === 'pg') { displayAmount = d.dose_amount * 1000000000; displayUnit = 'pg' }
    else if (doseUnit === 'mL') { displayAmount = d.dose_amount; displayUnit = 'mL' }
    return {
      dose_id: d.dose_id,
      drug_id: d.drug_id,
      dose_amount: d.dose_amount,
      display_amount: displayAmount,
      display_unit: displayUnit,
      timestamp: d.timestamp * 3600,
      route_of_administration: d.route,
      drugName: drugMap[d.drug_id] || d.drug_id,
    }
  })
}

export async function exportAllData() {
  const engine = await getEngine()

  const rawDoses = JSON.parse(JSON.stringify(engine.getAllDoses()))
  const rawDrugList = JSON.parse(JSON.stringify(engine.getAllDrugs()))
  const drugMap = {}
  for (const d of rawDrugList) { drugMap[d.drug_id] = d.name }

  const events = rawDoses.map(d => ({
    id: d.dose_id,
    route: d.route || 'oral',
    ester: drugMap[d.drug_id] || d.drug_id,
    timeH: d.timestamp,
    doseMG: d.dose_amount,
    extras: { drug_id: d.drug_id },
  }))

  return JSON.stringify({
    meta: { version: 1, exportedAt: new Date().toISOString() },
    weight: engine.getWeight(),
    events,
  }, null, 2)
}

export async function importAllData(jsonStr) {
  const engine = await getEngine()
  let data
  try { data = JSON.parse(jsonStr) } catch { throw new Error('Invalid JSON') }

  if (data.events && Array.isArray(data.events)) {
    for (const ev of data.events) {
      const drugId = resolveDrugId(engine, ev)
      if (!drugId) continue
      engine.addDose({
        dose_id: ev.id || crypto.randomUUID(),
        drug_id: drugId,
        dose_amount: ev.doseMG || 0,
        timestamp: ev.timeH || 0,
        route: ev.route || 'oral',
      })
    }
  }

  if (data.weight && !isNaN(data.weight) && data.weight > 0) {
    engine.setWeight(data.weight)
  }

  await saveAll(engine)
}

export async function setWeight(kg) {
  const engine = await getEngine()
  engine.setWeight(kg)
  await saveAll(engine)
}

export async function getWeight() {
  const engine = await getEngine()
  return engine.getWeight()
}

// ─── Engine Store ─────────────────────────────────────────────────
// Central service managing the ValenceEngine WASM instance.
// Handles: initialization, drug/dose CRUD, persistence, calibration, import/export.

import { getEngine as loadWasm } from './wasmLoader'
import {
  opfsWrite, opfsRead,
  restoreExternalDir, getExternalDirHandle,
  readExternalFile, writeExternalFile,
  pickDataDirectory, readFallback, writeFallback,
} from './persistence'
import type {
  DrugRecord, DoseRecord, DisplayDose,
  SimulationOutput, CalibrationBand,
  LabResult, DoseInput, DoseEvent, ExportPayload, RawDrugRecord,
} from './types'

// ─── Module State ─────────────────────────────────────────────────
let _engine: any = null
let _initPromise: Promise<any> | null = null

// Cache for drug metadata (dose_unit, routes) from preset JSON
const _drugMetaCache: Record<string, { dose_unit: string; routes: Array<{ route: string; unit: string }> }> = {}

// ─── Helpers ──────────────────────────────────────────────────────

/**
 * Get the server URL for a public/ asset.
 * Uses Vite's BASE_URL (empty in dev, '/reValenceGUI/' in prod).
 */
function assetUrl(path: string): string {
  const base = (import.meta as any).env?.BASE_URL
  // In dev: BASE_URL is '/' or undefined → files at root
  // In prod: BASE_URL is '/reValenceGUI/' → files under that path
  if (!base || base === '/') {
    return window.location.origin + '/' + path
  }
  return window.location.origin + base + path
}

function jsonDrugToRecord(d: RawDrugRecord): any {
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

function resolveDrugId(engine: any, ev: any): string | null {
  if (ev.extras?.drug_id) return ev.extras.drug_id

  const ester = ev.ester
  if (!ester) return null

  const allDrugs = JSON.parse(JSON.stringify(engine.getAllDrugs()))
  const byId = allDrugs.find((d: any) => d.drug_id === ester)
  if (byId) return ester

  const byName = allDrugs.find((d: any) => d.name === ester)
  if (byName) return byName.drug_id

  console.warn(`[engineStore] cannot resolve drug: "${ester}", skipping`)
  return null
}

// ─── Initialization ───────────────────────────────────────────────

export async function getEngine(): Promise<any> {
  if (_engine) return _engine
  if (!_initPromise) {
    _initPromise = initEngine()
  }
  await _initPromise
  return _engine
}

async function initEngine(): Promise<any> {
  const engine = await loadWasm()
  _engine = engine
  console.log('[engineStore] ValenceEngine initialized')

  // Restore previously selected external directory
  await restoreExternalDir()

  // Load user data (weight, calibration) from OPFS
  await loadUserDataFromOPFS(engine)

  // Migrate old separate weight storage
  const savedWeightRaw = readFallback('weight')
  if (savedWeightRaw) {
    const w = parseFloat(savedWeightRaw)
    if (!isNaN(w) && w > 0 && engine.getWeight() === 60) {
      engine.setWeight(w)
      await saveUserData(engine)
      try { localStorage.removeItem('valence_weight') } catch { /* ignore */ }
    }
  }

  // Load preset drugs
  await loadPresetDrugs(engine)

  // Load external or OPFS data
  const externalLoaded = getExternalDirHandle()
    ? await loadFromExternalDir(engine)
    : false

  if (!externalLoaded) {
    await loadCustomDrugsFromOPFS(engine)
    await loadDosesFromOPFS(engine)
  }

  if (getExternalDirHandle()) {
    await saveAll(engine)
  }

  return engine
}

// ─── Drug Presets ─────────────────────────────────────────────────

async function loadPresetDrugs(engine: any): Promise<void> {
  try {
    const [hrtResp, journalResp] = await Promise.all([
      fetch(assetUrl('data/hrt_drugs.json')),
      fetch(assetUrl('data/journal_drugs.json')),
    ])
    const hrt = await hrtResp.json()
    const journal = await journalResp.json()

    const all = [...hrt, ...journal].map((d: RawDrugRecord) => {
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

// ─── OPFS Load / Save ─────────────────────────────────────────────

async function loadCustomDrugsFromOPFS(engine: any): Promise<void> {
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

async function loadDosesFromOPFS(engine: any): Promise<void> {
  const raw = await opfsRead('doses.json')
  if (!raw) return

  let data: any
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

async function loadUserDataFromOPFS(engine: any): Promise<void> {
  const raw = await opfsRead('user.json')
  if (!raw) return
  try {
    engine.loadUserData(raw)
    console.log('[engineStore] loaded user data, weight:', engine.getWeight())
  } catch (e) {
    console.warn('[engineStore] failed to load user data:', e)
  }
}

export async function saveUserData(engine?: any): Promise<void> {
  if (!engine) engine = _engine
  if (!engine) return
  const json = engine.getUserData()
  await opfsWrite('user.json', json)
}

// ─── External Dir Load / Sync ─────────────────────────────────────

async function loadFromExternalDir(engine: any): Promise<boolean> {
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

// ─── Save All ─────────────────────────────────────────────────────

export async function saveAll(engine?: any): Promise<void> {
  if (!engine) engine = _engine
  if (!engine) return

  const rawDrugs = engine.getAllDrugs()
  const allDrugs = JSON.parse(JSON.stringify(rawDrugs))
  const customDrugs = allDrugs.filter((d: DrugRecord) =>
    !d.drug_id.startsWith('hrt_') && !d.drug_id.startsWith('journal_')
  )

  const rawDoses = JSON.parse(JSON.stringify(engine.getAllDoses()))
  const rawDrugList = JSON.parse(JSON.stringify(engine.getAllDrugs()))
  const drugMap: Record<string, string> = {}
  for (const d of rawDrugList) { drugMap[d.drug_id] = d.name }

  const events: DoseEvent[] = rawDoses.map((d: DoseRecord) => ({
    id: d.dose_id,
    route: d.route || 'oral',
    ester: drugMap[d.drug_id] || d.drug_id,
    timeH: d.timestamp,
    doseMG: d.dose_amount,
    extras: { drug_id: d.drug_id },
  }))

  const payload: ExportPayload = {
    meta: { version: 1, exportedAt: new Date().toISOString() },
    weight: engine.getWeight(),
    events,
  }

  await Promise.all([
    opfsWrite('custom_drugs.json', JSON.stringify(customDrugs, null, 2)),
    opfsWrite('doses.json', JSON.stringify(payload, null, 2)),
    saveUserData(engine),
  ])

  if (getExternalDirHandle()) {
    await syncToExternalDir(customDrugs, payload)
  }
}

async function syncToExternalDir(customDrugs: DrugRecord[], payload: ExportPayload): Promise<void> {
  try {
    await writeExternalFile('custom_drugs.json', JSON.stringify(customDrugs, null, 2))
    await writeExternalFile('doses.json', JSON.stringify(payload, null, 2))
    if (_engine) {
      await writeExternalFile('user.json', _engine.getUserData())
    }
    console.log('[engineStore] synced to external dir')
  } catch (e) {
    console.warn('[engineStore] sync failed:', e)
  }
}

// ─── Public API: Drugs ────────────────────────────────────────────

export async function getAllDrugsWithSource(): Promise<DrugRecord[]> {
  const engine = await getEngine()
  const raw = engine.getAllDrugs()
  const all: DrugRecord[] = JSON.parse(JSON.stringify(raw))

  for (const d of all) {
    if (d.drug_id.startsWith('hrt_')) d.source = 'hrt'
    else if (d.drug_id.startsWith('journal_')) d.source = 'journal'
    else d.source = 'custom'
    const meta = _drugMetaCache[d.drug_id]
    d.dose_unit = meta?.dose_unit || 'mg'
    d.routes = meta?.routes || [{ route: 'oral', unit: d.dose_unit }]
    d.parameters = d.params || {}
  }

  return all
}

export async function getCustomDrugs(): Promise<DrugRecord[]> {
  const all = await getAllDrugsWithSource()
  return all.filter(d => d.source === 'custom')
}

export async function getPresetDrugs(source?: string): Promise<DrugRecord[]> {
  const all = await getAllDrugsWithSource()
  return source
    ? all.filter(d => d.source === source)
    : all.filter(d => d.source !== 'custom')
}

export async function addDrug(drugData: RawDrugRecord): Promise<void> {
  const engine = await getEngine()
  _drugMetaCache[drugData.drug_id] = {
    dose_unit: drugData.dose_unit || 'mg',
    routes: drugData.routes || [{ route: 'oral', unit: drugData.dose_unit || 'mg' }],
  }
  engine.registerDrug(jsonDrugToRecord(drugData))
  await saveAll(engine)
}

export async function deleteDrug(drugId: string): Promise<void> {
  const engine = await getEngine()
  engine.removeDrug(drugId)
  delete _drugMetaCache[drugId]
  await saveAll(engine)
}

// ─── Public API: Doses ────────────────────────────────────────────

export async function addDose(doseData: DoseInput): Promise<void> {
  const engine = await getEngine()
  engine.addDose({
    dose_id: doseData.dose_id || crypto.randomUUID(),
    drug_id: doseData.drug_id,
    dose_amount: doseData.dose_amount || 0,
    timestamp: doseData.timestamp,
    route: doseData.route_of_administration || doseData.route || 'oral',
  })
  await saveAll(engine)
}

export async function removeDose(doseId: string): Promise<void> {
  const engine = await getEngine()
  engine.removeDose(doseId)
  await saveAll(engine)
}

export async function getAllDoses(): Promise<DisplayDose[]> {
  const engine = await getEngine()
  const raw = engine.getAllDoses()
  const doses: DoseRecord[] = JSON.parse(JSON.stringify(raw))

  const rawDrugs = engine.getAllDrugs()
  const drugMap: Record<string, string> = {}
  for (const d of JSON.parse(JSON.stringify(rawDrugs))) {
    drugMap[d.drug_id] = d.name
  }

  return doses.map(d => {
    const meta = _drugMetaCache[d.drug_id]
    const doseUnit = meta?.dose_unit || 'mg'
    let displayAmount = d.dose_amount
    let displayUnit = 'mg'
    if (doseUnit === 'µg') { displayAmount = d.dose_amount * 1000; displayUnit = 'µg' }
    else if (doseUnit === 'ng') { displayAmount = d.dose_amount * 1_000_000; displayUnit = 'ng' }
    else if (doseUnit === 'pg') { displayAmount = d.dose_amount * 1_000_000_000; displayUnit = 'pg' }
    else if (doseUnit === 'mL') { displayAmount = d.dose_amount; displayUnit = 'mL' }
    return {
      dose_id: d.dose_id,
      drug_id: d.drug_id,
      dose_amount: d.dose_amount,
      display_amount: displayAmount,
      display_unit: displayUnit,
      timestamp: d.timestamp * 3600, // hours → seconds
      route_of_administration: d.route,
      drugName: drugMap[d.drug_id] || d.drug_id,
    }
  })
}

// ─── Public API: Simulation ───────────────────────────────────────

export async function runSimulation(): Promise<SimulationOutput[]> {
  const engine = await getEngine()
  const raw = engine.runSimulation()
  return JSON.parse(JSON.stringify(raw))
}

export async function applyCalibration(sim: SimulationOutput): Promise<SimulationOutput | null> {
  const engine = await getEngine()
  const raw = engine.applyCalibration(sim)
  if (!raw) return null
  return JSON.parse(JSON.stringify(raw))
}

export async function getCalibrationBand(sim: SimulationOutput): Promise<CalibrationBand | null> {
  const engine = await getEngine()
  const raw = engine.getCalibrationBand(sim)
  if (!raw) return null
  return JSON.parse(JSON.stringify(raw))
}

// ─── Public API: Weight ───────────────────────────────────────────

export async function setWeight(kg: number): Promise<void> {
  const engine = await getEngine()
  engine.setWeight(kg)
  await saveUserData(engine)
}

export async function getWeight(): Promise<number> {
  const engine = await getEngine()
  return engine.getWeight()
}

// ─── Public API: Calibration ─────────────────────────────────────

export async function getCalibrationModel(): Promise<string> {
  const engine = await getEngine()
  return engine.getCalibrationModel()
}

export async function setCalibrationModel(model: string): Promise<void> {
  const engine = await getEngine()
  engine.setCalibrationModel(model)
  await saveUserData(engine)
}

export async function addLabResult(lab: LabResult): Promise<void> {
  const engine = await getEngine()
  engine.addLabResult(lab)
  await saveUserData(engine)
}

export async function clearLabResults(): Promise<void> {
  const engine = await getEngine()
  engine.clearLabResults()
  await saveUserData(engine)
}

export async function getLabResults(): Promise<LabResult[]> {
  const engine = await getEngine()
  return JSON.parse(JSON.stringify(engine.getLabResults()))
}

export async function getLabResultCount(): Promise<number> {
  const list = await getLabResults()
  return list.length
}

// ─── Public API: Import / Export ──────────────────────────────────

export async function exportAllData(): Promise<string> {
  const engine = await getEngine()
  const rawDoses: DoseRecord[] = JSON.parse(JSON.stringify(engine.getAllDoses()))
  const rawDrugList: DrugRecord[] = JSON.parse(JSON.stringify(engine.getAllDrugs()))
  const drugMap: Record<string, string> = {}
  for (const d of rawDrugList) { drugMap[d.drug_id] = d.name }

  const events: DoseEvent[] = rawDoses.map(d => ({
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

export async function importAllData(jsonStr: string): Promise<void> {
  const engine = await getEngine()
  let data: any
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

// ─── Public API: Engine Ready ─────────────────────────────────────

export function isEngineReady(): boolean {
  return _engine !== null
}

export async function waitForEngine(): Promise<any> {
  return getEngine()
}

// ─── WASM Module Type Definitions ──────────────────────────────────

export interface RouteInfo {
  route: string
  unit: string
}

export interface DrugRecord {
  drug_id: string
  name: string
  model_type: string
  group_id: string
  dose_unit: string
  routes: RouteInfo[]
  display_unit: string
  molecular_weight: number
  depot_model: boolean
  params: Record<string, number>
  parameters: Record<string, number>
  source?: 'hrt' | 'journal' | 'custom'
}

export interface DoseRecord {
  dose_id: string
  drug_id: string
  dose_amount: number
  timestamp: number
  route: string
}

export interface DisplayDose {
  dose_id: string
  drug_id: string
  dose_amount: number
  display_amount: number
  display_unit: string
  timestamp: number
  route_of_administration: string
  drugName: string
}

export interface SimulationOutput {
  time_h: number[]
  concentrations: number[]
  drug_name: string
  display_unit: string
}

export interface CalibrationBand {
  calibrated: number[]
  ci95_low: number[]
  ci95_high: number[]
  ci68_low: number[]
  ci68_high: number[]
}

export interface LabResult {
  id: string
  time_h: number
  conc_value: number
  unit: string
  group_id: string
}

export interface DoseInput {
  dose_id: string
  drug_id: string
  dose_amount: number
  timestamp: number
  route?: string
  route_of_administration?: string
}

export interface DoseEvent {
  id: string
  route: string
  ester: string
  timeH: number
  doseMG: number
  extras: { drug_id: string }
}

export interface ExportPayload {
  meta: { version: number; exportedAt: string }
  weight: number
  events: DoseEvent[]
}

// Raw DrugRecord from JSON (before engine processing)
export interface RawDrugRecord {
  drug_id: string
  name: string
  model_type?: string
  group_id?: string
  dose_unit?: string
  routes?: Array<{ route: string; unit: string }>
  display_unit?: string
  molecular_weight?: number
  depot_model?: boolean
  parameters?: Record<string, number>
}

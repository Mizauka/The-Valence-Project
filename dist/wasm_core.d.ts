/* tslint:disable */
/* eslint-disable */

export class DoseRecord {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    dose_amount: number;
    dose_id: string;
    drug_id: string;
    route: string;
    timestamp: number;
}

export class DrugRecord {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    bioavailability: number;
    clearance: number;
    display_unit: string;
    drug_id: string;
    equivalence_factor: number;
    half_life: number;
    k12: number;
    k21: number;
    ka: number;
    model_type: string;
    name: string;
    parent_compound: string;
    unit_conversion_factor: number;
    volume_of_distribution: number;
}

export class SimulationOutput {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    concentrations: Float64Array;
    display_unit: string;
    drug_name: string;
    time_h: Float64Array;
}

export class ValenceEngine {
    free(): void;
    [Symbol.dispose](): void;
    addDose(dose: any): void;
    clearDoses(): void;
    clearDrugs(): void;
    exportData(): any;
    getAllDoses(): any;
    getAllDrugs(): any;
    getDoseCount(): number;
    getDrug(drug_id: string): any;
    getDrugCount(): number;
    getWeight(): number;
    importData(json_str: string): void;
    constructor();
    registerDrug(drug: any): void;
    registerDrugs(drugs: any): void;
    removeDose(dose_id: string): boolean;
    removeDrug(drug_id: string): boolean;
    runSimulation(): any[];
    setWeight(kg: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_doserecord_free: (a: number, b: number) => void;
    readonly __wbg_drugrecord_free: (a: number, b: number) => void;
    readonly __wbg_get_doserecord_dose_amount: (a: number) => number;
    readonly __wbg_get_doserecord_dose_id: (a: number) => [number, number];
    readonly __wbg_get_doserecord_drug_id: (a: number) => [number, number];
    readonly __wbg_get_doserecord_route: (a: number) => [number, number];
    readonly __wbg_get_doserecord_timestamp: (a: number) => number;
    readonly __wbg_get_drugrecord_bioavailability: (a: number) => number;
    readonly __wbg_get_drugrecord_clearance: (a: number) => number;
    readonly __wbg_get_drugrecord_display_unit: (a: number) => [number, number];
    readonly __wbg_get_drugrecord_drug_id: (a: number) => [number, number];
    readonly __wbg_get_drugrecord_equivalence_factor: (a: number) => number;
    readonly __wbg_get_drugrecord_k12: (a: number) => number;
    readonly __wbg_get_drugrecord_k21: (a: number) => number;
    readonly __wbg_get_drugrecord_ka: (a: number) => number;
    readonly __wbg_get_drugrecord_model_type: (a: number) => [number, number];
    readonly __wbg_get_drugrecord_name: (a: number) => [number, number];
    readonly __wbg_get_drugrecord_parent_compound: (a: number) => [number, number];
    readonly __wbg_get_drugrecord_volume_of_distribution: (a: number) => number;
    readonly __wbg_get_simulationoutput_concentrations: (a: number) => [number, number];
    readonly __wbg_get_simulationoutput_display_unit: (a: number) => [number, number];
    readonly __wbg_get_simulationoutput_drug_name: (a: number) => [number, number];
    readonly __wbg_get_simulationoutput_time_h: (a: number) => [number, number];
    readonly __wbg_set_doserecord_dose_amount: (a: number, b: number) => void;
    readonly __wbg_set_doserecord_dose_id: (a: number, b: number, c: number) => void;
    readonly __wbg_set_doserecord_drug_id: (a: number, b: number, c: number) => void;
    readonly __wbg_set_doserecord_route: (a: number, b: number, c: number) => void;
    readonly __wbg_set_doserecord_timestamp: (a: number, b: number) => void;
    readonly __wbg_set_drugrecord_bioavailability: (a: number, b: number) => void;
    readonly __wbg_set_drugrecord_clearance: (a: number, b: number) => void;
    readonly __wbg_set_drugrecord_display_unit: (a: number, b: number, c: number) => void;
    readonly __wbg_set_drugrecord_drug_id: (a: number, b: number, c: number) => void;
    readonly __wbg_set_drugrecord_equivalence_factor: (a: number, b: number) => void;
    readonly __wbg_set_drugrecord_k12: (a: number, b: number) => void;
    readonly __wbg_set_drugrecord_k21: (a: number, b: number) => void;
    readonly __wbg_set_drugrecord_ka: (a: number, b: number) => void;
    readonly __wbg_set_drugrecord_model_type: (a: number, b: number, c: number) => void;
    readonly __wbg_set_drugrecord_name: (a: number, b: number, c: number) => void;
    readonly __wbg_set_drugrecord_parent_compound: (a: number, b: number, c: number) => void;
    readonly __wbg_set_drugrecord_volume_of_distribution: (a: number, b: number) => void;
    readonly __wbg_set_simulationoutput_concentrations: (a: number, b: number, c: number) => void;
    readonly __wbg_set_simulationoutput_display_unit: (a: number, b: number, c: number) => void;
    readonly __wbg_set_simulationoutput_drug_name: (a: number, b: number, c: number) => void;
    readonly __wbg_set_simulationoutput_time_h: (a: number, b: number, c: number) => void;
    readonly __wbg_simulationoutput_free: (a: number, b: number) => void;
    readonly __wbg_valenceengine_free: (a: number, b: number) => void;
    readonly valenceengine_addDose: (a: number, b: any) => [number, number];
    readonly valenceengine_clearDoses: (a: number) => void;
    readonly valenceengine_clearDrugs: (a: number) => void;
    readonly valenceengine_exportData: (a: number) => any;
    readonly valenceengine_getAllDoses: (a: number) => any;
    readonly valenceengine_getAllDrugs: (a: number) => any;
    readonly valenceengine_getDoseCount: (a: number) => number;
    readonly valenceengine_getDrug: (a: number, b: number, c: number) => any;
    readonly valenceengine_getDrugCount: (a: number) => number;
    readonly valenceengine_getWeight: (a: number) => number;
    readonly valenceengine_importData: (a: number, b: number, c: number) => [number, number];
    readonly valenceengine_new: () => number;
    readonly valenceengine_registerDrug: (a: number, b: any) => [number, number];
    readonly valenceengine_registerDrugs: (a: number, b: any) => [number, number];
    readonly valenceengine_removeDose: (a: number, b: number, c: number) => number;
    readonly valenceengine_removeDrug: (a: number, b: number, c: number) => number;
    readonly valenceengine_runSimulation: (a: number) => [number, number];
    readonly valenceengine_setWeight: (a: number, b: number) => void;
    readonly __wbg_get_drugrecord_half_life: (a: number) => number;
    readonly __wbg_get_drugrecord_unit_conversion_factor: (a: number) => number;
    readonly __wbg_set_drugrecord_half_life: (a: number, b: number) => void;
    readonly __wbg_set_drugrecord_unit_conversion_factor: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __externref_drop_slice: (a: number, b: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;

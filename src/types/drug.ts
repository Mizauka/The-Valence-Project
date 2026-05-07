export interface DrugRoute {
  route: string
  unit: string
}

export interface DrugParameters {
  half_life: number
  volume_of_distribution?: number
  ka?: number
  bioavailability?: number
  equivalence_factor?: number
  k_clear?: number
}

export interface Drug {
  drug_id: string
  name: string
  model_type: string
  dose_unit: string
  source?: string
  routes?: DrugRoute[]
  parameters?: DrugParameters
}

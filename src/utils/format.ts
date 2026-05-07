// ── Time formatting ───────────────────────────────────────

export function fmtTime(ms: number): string {
  const d = new Date(ms)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${pad(d.getMonth() + 1)}/${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

export function fmtHours(th: number): string {
  return new Date(th * 3600000).toLocaleString()
}

export function getCurrentTimestamp(): string {
  const now = new Date()
  const offset = now.getTimezoneOffset()
  const local = new Date(now.getTime() - offset * 60000)
  return local.toISOString().slice(0, 16)
}

// ── Concentration formatting ─────────────────────────────

export function fmtTooltip(val: number, unit: string): string {
  if (!isFinite(val)) return '\u2014'
  switch (unit) {
    case 'pg/mL': return `${Math.round(val).toLocaleString()} ${unit}`
    case 'ng/mL': return `${val.toFixed(2)} ${unit}`
    case 'µg/mL': return `${val.toFixed(2)} ${unit}`
    case 'mg/L': return `${val.toFixed(4)} ${unit}`
    default: return `${val.toFixed(4)} ${unit}`
  }
}

// ── Dose formatting ──────────────────────────────────────

export function formatDose(val: number, unit: string): string {
  if (Number.isInteger(val) || Math.abs(val) >= 10) return `${val} ${unit}`
  if (Math.abs(val) >= 1) return `${val.toFixed(1)} ${unit}`
  return `${val.toFixed(2)} ${unit}`
}

// ── Route labels ─────────────────────────────────────────

export const ROUTE_LABELS: Record<string, string> = {
  oral: '口服', injection: '注射', sublingual: '舌下', buccal: '颊黏膜',
  insufflated: '鼻吸', transdermal: '透皮', gel: '凝胶', patch: '贴片',
  rectal: '直肠', smoked: '吸入(烟)', inhaled: '吸入', inhalation: '吸入',
}

export function routeLabel(key: string): string {
  return ROUTE_LABELS[key] || key
}

// ── Model labels ─────────────────────────────────────────

export function modelLabel(modelType: string): string {
  const map: Record<string, string> = {
    one_compartment: '一室', two_compartment: '二室', multi_compartment: '多室',
  }
  return map[modelType] || modelType
}

// ── File download ────────────────────────────────────────

export function downloadBlob(content: string, filename: string, mime: string): void {
  const blob = new Blob([content], { type: mime })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url; a.download = filename; a.click()
  URL.revokeObjectURL(url)
}

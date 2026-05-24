import { ref, onMounted, nextTick, type Directive, type Ref } from 'vue'

type LitPropValue = string | number | boolean | string[] | number[]
type LitPropsMap = Record<string, LitPropValue>

// 存储每个元素的上一份 props 快照，避免无谓的重复更新
const _snapshots = new WeakMap<HTMLElement, string>()

function snapshotKey(props: LitPropsMap | undefined): string {
  if (!props) return ''
  return Object.entries(props)
    .sort(([a], [b]) => (a < b ? -1 : 1))
    .map(([k, v]) => `${k}:${Array.isArray(v) ? v.join(',') : String(v)}`)
    .join('|')
}

export const vLit: Directive<HTMLElement, LitPropsMap | undefined> = {
  created(el, binding) {
    apply(el, binding.value)
    _snapshots.set(el, snapshotKey(binding.value))
  },
  updated(el, binding) {
    const key = snapshotKey(binding.value)
    if (_snapshots.get(el) === key) return
    _snapshots.set(el, key)
    apply(el, binding.value)
  },
}

function apply(el: HTMLElement, props: LitPropsMap | undefined): void {
  if (!props) return
  for (const [name, value] of Object.entries(props)) {
    if (typeof value === 'boolean') {
      value ? el.setAttribute(name, '') : el.removeAttribute(name)
    } else if (Array.isArray(value)) {
      el.setAttribute(name, value.join(' '))
    } else {
      el.setAttribute(name, String(value))
    }
  }
  ;(el as any).requestUpdate?.()
}

// ─── Composable 方式（:ref 函数回调） ─────────────────────
//
// 用法:
//   const { elRef } = useLitProps({ handle: true })
//   <m3e-bottom-sheet :ref="elRef">

export function useLitProps(props: LitPropsMap): {
  elRef: (el: unknown) => void
} {
  return {
    elRef: (el: unknown) => {
      if (!el) return
      apply(el as HTMLElement, props)
    },
  }
}

export function useLitPropsMounted(props: LitPropsMap): {
  elRef: Ref<HTMLElement | null>
} {
  const elRef = ref<HTMLElement | null>(null)
  onMounted(async () => {
    await nextTick()
    if (elRef.value) apply(elRef.value, props)
  })
  return { elRef }
}

import { ref, computed, onMounted, onUnmounted } from 'vue'

const MOBILE_BP = 768

/**
 * Generic multi-step sliding panel composable.
 *
 * Desktop (>=768px): show 2 panels side-by-side.
 *   (0,1) → (1,2) → … → (N-2, N-1)
 *
 * Mobile (<768px): show 1 panel at a time.
 *   0 → 1 → 2 → … → N-1
 *
 * `advance(fromPanel)` only succeeds when called from the rightmost
 * visible panel (or when step===0, which allows the first transition).
 *
 * CSS requirements:
 *   .viewport { overflow: hidden; }
 *   .track { display: flex; height: 100%; transition: transform .35s; }
 *   .track > * { flex: 1; min-width: 0; }
 */
export function useSlidingPanels(totalPanels: number) {
  const step = ref(0)
  const isMobile = ref(false)

  function checkMobile() {
    isMobile.value = window.innerWidth < MOBILE_BP
  }

  onMounted(() => {
    checkMobile()
    window.addEventListener('resize', checkMobile)
  })
  onUnmounted(() => {
    window.removeEventListener('resize', checkMobile)
  })

  // ── derived ──────────────────────────────────────────────

  const maxStep = computed(() =>
    isMobile.value ? totalPanels - 1 : totalPanels - 2
  )

  /** Which panel is the rightmost visible one? */
  const rightmostPanel = computed(() =>
    isMobile.value ? step.value : step.value + 1
  )

  /** Track width as percentage of viewport. */
  const trackWidthPercent = computed(() =>
    isMobile.value ? totalPanels * 100 : (totalPanels / 2) * 100
  )

  /** Horizontal offset of the track relative to viewport (%). */
  const offsetPercent = computed(() => {
    const onePanelPct = 100 / totalPanels
    return step.value * onePanelPct * (isMobile.value ? 1 : 1)
    // Desktop: step 0→0%, step 1→25%, …  (each step = 1 panel = 100/N %)
    // Mobile:  step 0→0%, step 1→20%, …  (1 panel per step)
  })

  // ── actions ──────────────────────────────────────────────

  /**
   * Advance one step.
   * @param fromPanel  Which panel index triggered this call.
   *                   Only honoured when it matches the rightmost visible panel,
   *                   unless step===0 (first transition always allowed).
   */
  function advance(fromPanel?: number) {
    if (step.value >= maxStep.value) return
    // Allow if fromPanel matches the rightmost, or if we're at step 0
    if (fromPanel !== undefined && fromPanel !== rightmostPanel.value && step.value !== 0) return
    step.value++
  }

  function back() {
    if (step.value > 0) step.value--
  }

  function reset() {
    step.value = 0
  }

  return {
    step,
    isMobile,
    maxStep,
    rightmostPanel,
    trackWidthPercent,
    offsetPercent,
    advance,
    back,
    reset,
  }
}


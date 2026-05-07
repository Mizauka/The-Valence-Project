import { ref, computed, reactive, onMounted, onUnmounted } from 'vue'

const MOBILE_BP = 768

export function useSlidingPanels(totalPanels: number, placeholder?: { icon: string; text: string }) {
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

  const maxStep = computed(() =>
    isMobile.value ? totalPanels - 1 : totalPanels - 2
  )

  const trackWidthPercent = computed(() =>
    isMobile.value ? totalPanels * 100 : totalPanels * 50
  )

  const offsetPercent = computed(() =>
    isMobile.value ? step.value * 100 : step.value * 50
  )

  const _hasSelection = ref(false)
  const _animating = ref(false)

  function markSelection() { _hasSelection.value = true }

  const showPlaceholder = computed(() => {
    if (_hasSelection.value || _animating.value) return false
    return placeholder !== undefined
  })

  function advance(fromPanel?: number) {
    if (step.value >= maxStep.value) return
    step.value++
  }

  function reset() {
    step.value = 0
    _hasSelection.value = false
  }

  function back(): boolean {
    if (step.value > 0) {
      _animating.value = true
      step.value--
      _hasSelection.value = false
      setTimeout(() => { _animating.value = false }, 370)
      return true
    }
    return false
  }

  const placeholderIcon = placeholder?.icon || 'edit_note'
  const placeholderText = placeholder?.text || '在左侧选择后在此编辑'

  return reactive({
    get step() { return step.value },
    get isMobile() { return isMobile.value },
    get maxStep() { return maxStep.value },
    get trackWidthPercent() { return trackWidthPercent.value },
    get offsetPercent() { return offsetPercent.value },
    get showPlaceholder() { return showPlaceholder.value },
    placeholderIcon,
    placeholderText,
    advance,
    back,
    reset,
    markSelection,
  })
}

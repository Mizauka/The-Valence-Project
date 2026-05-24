import { computed, ref } from "vue"
import { useBreakpoint } from "./useBreakpoint"
import { useParentWidth } from "./useParentWidth"

export interface SplitPaneOptions {
  leftFraction?: number
  leftMin?: number
  rightFraction?: number
  rightMin?: number
  initValue?: number
}

export function useSplitPane(opts: SplitPaneOptions = {}) {
  const {
    leftFraction = 0.25,
    leftMin = 170,
    rightFraction = 0.5,
    rightMin = 340,
    initValue,
  } = opts

  const { width } = useBreakpoint()
  const paneRef = ref<HTMLElement | null>(null)
  const parentWidth = useParentWidth(paneRef, 0)

  const minLeftPaneWidth = computed(
    () => `${Math.max(parentWidth.value * leftFraction - 12, leftMin)}px`,
  )
  const minRightPaneWidth = computed(
    () => `${Math.max(parentWidth.value * rightFraction - 12, rightMin)}px`,
  )

  const splitProps = computed(() => {
    const w = width.value
    const minL = Math.max(parentWidth.value * leftFraction - 12, leftMin)
    const minR = Math.max(parentWidth.value * rightFraction - 12, rightMin)
    const coefficient = w < 900
      ? Math.min((minL + minR + 24) / parentWidth.value, 1)
      : 1
    const value = initValue != null
      ? w < 900 ? initValue * coefficient : initValue
      : w < 900 ? 50 * coefficient : 25
    return { value, detents: [0, 25, 50 * coefficient, 100], wrapDetents: true }
  })

  return { width, paneRef, parentWidth, minLeftPaneWidth, minRightPaneWidth, splitProps }
}

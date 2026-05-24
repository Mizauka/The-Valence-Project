import { ref, readonly, onMounted, onUnmounted } from 'vue'

/**
 * nav-rail 的当前模式（expanded / compact），全局共享
 */
const _isNavExpanded = ref(false)
let _resizeObserver: ResizeObserver | null = null

export function useNavRailMode() {
  const navRef = ref<HTMLElement | null>(null)

  onMounted(() => {
    const rail = navRef.value
    if (!rail) return
    _isNavExpanded.value = rail.clientWidth > 100

    _resizeObserver = new ResizeObserver(() => {
      _isNavExpanded.value = rail.clientWidth > 100
    })
    _resizeObserver.observe(rail)
  })

  onUnmounted(() => {
    _resizeObserver?.disconnect()
  })

  return {
    navRef,
    isNavExpanded: readonly(_isNavExpanded),
  }
}

import { ref, onMounted, onUnmounted } from 'vue'

const innerWidth = ref(window.innerWidth)
const isPhone = ref(false)
const isTablet = ref(false)
const isDesktop = ref(false)

let previousBreakpoint: string | null = null
let listenerCount = 0

function update() {
  const w = window.innerWidth
  innerWidth.value = w
  let currentBreakpoint: string
  if (w < 640) currentBreakpoint = 'phone'
  else if (w < 1200) currentBreakpoint = 'tablet'
  else currentBreakpoint = 'desktop'
  if (currentBreakpoint === previousBreakpoint) return
  previousBreakpoint = currentBreakpoint
  isPhone.value = currentBreakpoint === 'phone'
  isTablet.value = currentBreakpoint === 'tablet'
  isDesktop.value = currentBreakpoint === 'desktop'
}

export function useBreakpoint() {
  onMounted(() => {
    if (listenerCount++ === 0) {
      update()
      window.addEventListener('resize', update)
    }
  })
  onUnmounted(() => {
    if (--listenerCount === 0) {
      window.removeEventListener('resize', update)
    }
  })
  return { width: innerWidth, isPhone, isTablet, isDesktop }
}

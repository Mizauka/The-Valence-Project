import { ref, watch, onMounted, onUnmounted, type Ref } from 'vue'

/**
 * 监听指定元素（或其祖先）的宽度。
 *
 * @param elRef   目标元素的 template ref
 * @param level   向上查找的层级（0 = 自身，1 = 父元素，默认 1）
 * @returns       宽度的响应式 ref（px）
 */
export function useParentWidth(
  elementRef: Ref<HTMLElement | null>,
  level = 1,
): Ref<number> {
  const width = ref(0)
  let observer: ResizeObserver | null = null

  const observe = () => {
    const el = elementRef.value
    if (!el) return

    let parent: HTMLElement | null = el
    for (let i = 0; i < level && parent; i++) {
      parent = parent.parentElement
    }
    if (!parent) return

    observer?.disconnect()
    width.value = parent.clientWidth

    observer = new ResizeObserver(([entry]) => {
      width.value =
        entry?.borderBoxSize?.[0]?.inlineSize ??
        entry?.contentRect?.width ??
        0
    })
    observer.observe(parent)
  }

  // 处理 v-if 延迟挂载
  watch(elementRef, (el) => {
    if (el) requestAnimationFrame(observe)
  }, { immediate: true })

  onUnmounted(() => {
    observer?.disconnect()
  })

  return width
}

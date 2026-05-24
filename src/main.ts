import { createApp } from 'vue'
import App from './App.vue'
import router, { BASE } from './router'
import 'mdui/mdui.css';
import 'mdui';
import "@m3e/web/all";
import 'material-symbols';
import 'material-icons';
import './style.css';

const app = createApp(App)

app.use(router)

// GitHub Pages SPA 回退恢复：404.html → 存 sessionStorage → 回到入口 → 恢复到原始路由
router.isReady().then(() => {
  const redirect = sessionStorage.getItem('spa-redirect')
  if (redirect) {
    sessionStorage.removeItem('spa-redirect')
    const path = redirect.replace(BASE, '/').replace(/\/+$/, '') || '/'
    router.replace(path)
  }
})

// 给所有 m3e-icon 加上 translate="no" 防止浏览器翻译破坏连字图标
const observer = new MutationObserver(() => {
  document.querySelectorAll('m3e-icon, m3e-icon-button').forEach(el => {
    if (!el.hasAttribute('translate')) el.setAttribute('translate', 'no')
  })
})
observer.observe(document.documentElement, { childList: true, subtree: true })

app.mount('#app')

import { createApp } from 'vue'
import 'mdui/mdui.css'
import 'mdui'
import '@fontsource/material-icons-outlined/400.css'
import '@fontsource/material-icons/400.css'
import './style.css'
import App from './App.vue'
import router from './router'

createApp(App).use(router).mount('#app')

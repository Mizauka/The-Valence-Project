import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vite";
import vueDevTools from "vite-plugin-vue-devtools";
import vue from "@vitejs/plugin-vue";

export default defineConfig(({ mode }) => ({
  // GitHub Pages 部署基础路径（仓库名）。若使用自定义域名或 username.github.io 仓库，改为 '/'
  base: mode === 'production' ? '/The-Valence-Project/' : '/',
  server: {
    host: true,
  },
  // 防止 Lit 实例重复（Vite 预打包时可能创建多份 lit 运行时）
  resolve: {
    dedupe: ["lit", "lit-html", "@lit/reactive-element"],
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  plugins: [
    ...(mode !== 'production' ? [vueDevTools()] : []),
    vue({
      template: {
        compilerOptions: {
          // 所有以 mdui- 或 m3e- 开头的标签名都是 web component
          isCustomElement: (tag) =>
            tag.startsWith("mdui-") || tag.startsWith("m3e-"),
        },
      },
    }),
  ],
}));

# 前端 WASM 加载器 — wasmIntegration.js

## 文件位置
`src/wasm/wasmIntegration.js`

## 概述
负责动态加载 WebAssembly 模块。独立于 `engineStore.js` 存在，提供更底层的加载控制。

> **注意**：当前项目主要使用 `engineStore.js` 中的集成初始化流程。此文件保留作为备用的底层加载器。

## 模块级变量

| 变量 | 说明 |
|------|------|
| `engine` | ValenceEngine 实例 |
| `initPromise` | 初始化 Promise（防重入） |
| `loadError` | 缓存加载错误 |

## 导出函数

### `getEngine() → Promise<ValenceEngine>`
获取引擎实例，未初始化时自动初始化。如果之前初始化失败，直接抛出缓存的错误。

### `isLoaded() → boolean`
检查引擎是否已加载。

### `getLoadError() → Error | null`
获取加载错误信息。

## 初始化流程 `_init()`

```
1. fetch /wasm/wasm_core.js（JS 胶水代码）
2. 将 JS 代码转为 Blob URL
3. 动态 import Blob URL
4. 调用模块的 default 函数初始化 wasm-bindgen
5. 从模块导出中获取 ValenceEngine 类
6. 实例化引擎
7. 验证实例有效性
```

## 与 engineStore.js 的关系

`engineStore.js` 是主用入口，`wasmIntegration.js` 是备选底层加载器。两者的加载逻辑基本一致，区别在于：
- `engineStore` 额外处理 OPFS、IndexedDB、数据持久化
- `wasmIntegration` 仅负责最基础的 WASM 加载和实例化

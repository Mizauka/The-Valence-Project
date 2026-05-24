export interface RouteMeta {
  icon: string
  title: string
  name: string
}

export const routeMeta: Record<string, RouteMeta> = {
  home: { name: 'home', icon: 'dataset', title: '浓度曲线' },
  history: { name: 'history', icon: 'watch_later', title: '药历核对' },
  library: { name: 'library', icon: 'medication', title: '物质列表' },
  plan: { name: 'plan', icon: 'clinical_notes', title: '用药方案' },
  add: { name: 'add', icon: 'add', title: '添加记录' },
  settings: { name: 'settings', icon: 'settings', title: '设置' },
}

export const navItems = Object.values(routeMeta)

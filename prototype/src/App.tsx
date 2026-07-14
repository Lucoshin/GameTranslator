import { useState } from 'react'
import { BookWorkspace } from './components/BookWorkspace'
import { Icon } from './components/Icon'
import { ProjectCenter } from './components/ProjectCenter'

const productNav = [
  ['projects', '项目'],
  ['glossary', '公共术语库'],
  ['settings', '模型与设置'],
  ['history', '历史'],
] as const

export default function App() {
  const [view, setView] = useState<'projects' | 'book'>('projects')

  return (
    <div className="app-shell">
      <nav aria-label="产品导航" className="product-nav">
        <button type="button" className="brand-button" aria-label="GameTranslator 首页" onClick={() => setView('projects')}><span>译</span></button>
        {productNav.map(([icon, label]) => (
          <button type="button" className={label === '项目' ? 'active' : ''} key={label} onClick={() => label === '项目' && setView('projects')}>
            <Icon name={icon} />
            <span>{label}</span>
          </button>
        ))}
        <div className="nav-spacer" />
        <button type="button" className="nav-avatar" aria-label="当前用户">L</button>
      </nav>
      {view === 'projects'
        ? <ProjectCenter onOpenBook={() => setView('book')} />
        : <BookWorkspace />}
    </div>
  )
}

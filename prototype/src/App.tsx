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
      <nav aria-label="产品导航">
        <button type="button" aria-label="GameTranslator 首页" onClick={() => setView('projects')}>译</button>
        {productNav.map(([icon, label]) => (
          <button type="button" key={label} onClick={() => label === '项目' && setView('projects')}>
            <Icon name={icon} />
            <span>{label}</span>
          </button>
        ))}
      </nav>
      {view === 'projects'
        ? <ProjectCenter onOpenBook={() => setView('book')} />
        : <BookWorkspace />}
    </div>
  )
}

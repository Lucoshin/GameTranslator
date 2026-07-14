import { useRef, useState } from 'react'
import { demoBook, initialSegments } from '../data/demoBook'
import { ChapterTree } from './ChapterTree'
import { Inspector } from './Inspector'
import { ReadingEditor } from './ReadingEditor'
import { SegmentReview } from './SegmentReview'

const navItems = ['书稿', '编辑', '问题', '术语', '导出']

export function BookWorkspace() {
  const [mode, setMode] = useState<'reading' | 'review'>('reading')
  const [segments, setSegments] = useState(initialSegments)
  const [selectedId, setSelectedId] = useState(1)
  const [saveState, setSaveState] = useState<'saved' | 'saving'>('saved')
  const saveTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)
  const selectedSegment = segments.find((segment) => segment.id === selectedId) ?? segments[0]

  const updateTranslation = (id: number, value: string) => {
    setSelectedId(id)
    setSegments((current) => current.map((segment) => segment.id === id ? { ...segment, translation: value } : segment))
    setSaveState('saving')
    clearTimeout(saveTimer.current)
    saveTimer.current = setTimeout(() => setSaveState('saved'), 300)
  }

  const confirmSegment = (id: number) => {
    setSegments((current) => current.map((segment) => segment.id === id ? { ...segment, status: 'reviewed' } : segment))
    const index = segments.findIndex((segment) => segment.id === id)
    const next = segments[index + 1]
    if (next) setSelectedId(next.id)
  }

  return (
    <main className="book-workspace">
      <header className="workspace-header">
        <div className="book-identity">
          <span className="book-mark">B</span>
          <div><p>BOOK / 书籍项目</p><h1>{demoBook.title}</h1></div>
        </div>
        <div className="workspace-actions">
          <span className={`save-state ${saveState}`}>{saveState === 'saving' ? '正在保存…' : '已保存'}</span>
          <button type="button" className="model-chip"><i />deepseek-v4-flash</button>
          <button type="button" className="primary-action">翻译本章</button>
        </div>
      </header>
      <nav aria-label="书籍项目导航" className="workspace-nav">
        {navItems.map((item) => <button type="button" className={item === '编辑' ? 'active' : ''} key={item}>{item}{item === '问题' && <b>9</b>}</button>)}
      </nav>
      <div className="editor-toolbar">
        <div><strong>第八章 · 潮声背后</strong><span>5 个段落 · 2 个待处理问题</span></div>
        <div className="mode-switch" aria-label="编辑模式">
          <button type="button" aria-pressed={mode === 'reading'} onClick={() => setMode('reading')}>阅读编辑</button>
          <button type="button" aria-pressed={mode === 'review'} onClick={() => setMode('review')}>逐段校对</button>
        </div>
      </div>
      <div className="workspace-grid">
        <ChapterTree />
        <div className="editor-stage">
          {mode === 'reading'
            ? <ReadingEditor segments={segments} selectedId={selectedId} onSelect={setSelectedId} onChange={updateTranslation} onConfirm={confirmSegment} />
            : <SegmentReview segments={segments} selectedId={selectedId} onSelect={setSelectedId} onChange={updateTranslation} onConfirm={confirmSegment} />}
        </div>
        <Inspector segment={selectedSegment} />
      </div>
      <footer className="status-bar">
        <span><i className="online" /> 本地项目 · 自动保存</span>
        <span>第 8 / 12 章</span>
        <span>本章 1,284 字</span>
        <span className="shortcut"><kbd>Ctrl</kbd> + <kbd>Enter</kbd> 确认并继续</span>
      </footer>
    </main>
  )
}

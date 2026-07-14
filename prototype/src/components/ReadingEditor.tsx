import type { ChangeEvent, KeyboardEvent } from 'react'
import type { BookSegment } from '../data/demoBook'

type EditorProps = {
  segments: BookSegment[]
  selectedId: number
  onSelect: (id: number) => void
  onChange: (id: number, value: string) => void
  onConfirm: (id: number) => void
}

export function ReadingEditor({ segments, selectedId, onSelect, onChange, onConfirm }: EditorProps) {
  const handleKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>, id: number) => {
    if (event.ctrlKey && event.key === 'Enter') {
      event.preventDefault()
      onConfirm(id)
    }
  }

  return (
    <section className="reading-page" aria-label="阅读编辑">
      <header className="chapter-title">
        <span>CHAPTER EIGHT</span>
        <h2>潮声背后</h2>
        <p>第八章</p>
      </header>
      <div className="manuscript">
        {segments.map((segment) => (
          <article key={segment.id} className={`paragraph ${selectedId === segment.id ? 'selected' : ''}`}>
            <button
              type="button"
              className={`status-dot ${segment.status}`}
              aria-label={`选择段落 ${segment.id}`}
              aria-current={selectedId === segment.id ? 'true' : undefined}
              onClick={() => onSelect(segment.id)}
            />
            <span className="sr-only">段落 {segment.id} 状态：{segment.status === 'reviewed' ? '已校对' : segment.status === 'issue' ? '有问题' : 'AI 草稿'}</span>
            {selectedId === segment.id && <p className="source-preview" lang="en">{segment.source}</p>}
            <textarea
              aria-label={`段落 ${segment.id} 译文`}
              value={segment.translation}
              rows={Math.max(2, Math.ceil(segment.translation.length / 35))}
              onFocus={() => onSelect(segment.id)}
              onChange={(event: ChangeEvent<HTMLTextAreaElement>) => onChange(segment.id, event.target.value)}
              onKeyDown={(event) => handleKeyDown(event, segment.id)}
            />
          </article>
        ))}
      </div>
    </section>
  )
}

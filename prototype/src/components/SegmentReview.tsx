import type { KeyboardEvent } from 'react'
import type { BookSegment } from '../data/demoBook'

type ReviewProps = {
  segments: BookSegment[]
  selectedId: number
  onSelect: (id: number) => void
  onChange: (id: number, value: string) => void
  onConfirm: (id: number) => void
}

export function SegmentReview({ segments, selectedId, onSelect, onChange, onConfirm }: ReviewProps) {
  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>, id: number) => {
    if (event.ctrlKey && event.key === 'Enter') {
      event.preventDefault()
      onConfirm(id)
    }
  }

  return (
    <section className="segment-review" aria-label="逐段校对">
      <div className="review-columns"><span>原文</span><span>译文</span></div>
      {segments.map((segment) => (
        <div
          key={segment.id}
          className={`review-row ${selectedId === segment.id ? 'selected' : ''}`}
        >
          <button type="button" className="row-index" aria-label={`选择段落 ${segment.id}`} aria-current={selectedId === segment.id ? 'true' : undefined} onClick={() => onSelect(segment.id)}>{String(segment.id).padStart(2, '0')}</button>
          <span lang="en" className="source-cell">{segment.source}</span>
          <textarea
            aria-label={`段落 ${segment.id} 译文`}
            value={segment.translation}
            onFocus={() => onSelect(segment.id)}
            onChange={(event) => onChange(segment.id, event.target.value)}
            onKeyDown={(event) => onKeyDown(event, segment.id)}
          />
        </div>
      ))}
    </section>
  )
}

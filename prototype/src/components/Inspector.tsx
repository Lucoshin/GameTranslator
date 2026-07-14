import type { BookSegment } from '../data/demoBook'

const statusLabels = { draft: 'AI 草稿', reviewed: '已校对', issue: '有问题' }

export function Inspector({ segment }: { segment: BookSegment }) {
  return (
    <aside className="inspector" aria-label="段落检查器">
      <div className="panel-heading">
        <div>
          <span className="eyebrow">INSPECTOR</span>
          <h2>当前段落 {segment.id}</h2>
        </div>
        <span className={`status-pill ${segment.status}`}>{statusLabels[segment.status]}</span>
      </div>
      <section>
        <h3>原文</h3>
        <p lang="en" className="inspector-source">{segment.source}</p>
      </section>
      <section>
        <h3>质量检查</h3>
        {segment.note
          ? <div className="qa-note"><strong>译文表达</strong><p>{segment.note}</p></div>
          : <div className="qa-clear"><strong>未发现阻断问题</strong><p>控制结构和标点检查已通过。</p></div>}
      </section>
      <section>
        <h3>命中术语</h3>
        {segment.terms?.length
          ? segment.terms.map((term) => <span className="term-chip" key={term}>{term}</span>)
          : <p className="muted">当前段落没有命中术语。</p>}
      </section>
      <section className="ai-suggestion">
        <div><h3>AI 建议</h3><span>deepseek-v4-flash</span></div>
        <p>保持当前叙述节奏，减少连续形容词，让句子更接近中文小说的呼吸感。</p>
        <button type="button">重新生成建议</button>
      </section>
    </aside>
  )
}

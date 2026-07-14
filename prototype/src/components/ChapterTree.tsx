import { chapters } from '../data/demoBook'

export function ChapterTree() {
  return (
    <aside className="chapter-panel" aria-label="章节目录">
      <div className="panel-heading">
        <div>
          <span className="eyebrow">CONTENTS</span>
          <h2>书稿目录</h2>
        </div>
        <span className="chapter-count">12 章</span>
      </div>
      <div className="chapter-progress">
        <span>全书进度</span><strong>68%</strong>
        <div><i style={{ width: '68%' }} /></div>
      </div>
      <ol className="chapter-list">
        {chapters.map((chapter) => (
          <li key={chapter.number}>
            <button type="button" className={chapter.active ? 'active' : ''} aria-current={chapter.active ? 'page' : undefined}>
              <span className="chapter-number">{chapter.number}</span>
              <span className="chapter-copy">
                <strong>{chapter.title}</strong>
                <span>{chapter.progress}% 已校对{chapter.issues ? ` · ${chapter.issues} 个问题` : ''}</span>
              </span>
              {chapter.issues > 0 && <b className="issue-count">{chapter.issues}</b>}
            </button>
          </li>
        ))}
      </ol>
    </aside>
  )
}

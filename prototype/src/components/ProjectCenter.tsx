import { demoBook } from '../data/demoBook'

export function ProjectCenter({ onOpenBook }: { onOpenBook: () => void }) {
  return (
    <main className="project-center">
      <header className="project-center-header">
        <div><p className="eyebrow">PROJECT LIBRARY / 项目中心</p><h1>所有项目</h1><p>在同一个工作室里，继续你的游戏汉化与书籍译校。</p></div>
        <button type="button" className="primary-action">＋ 新建项目</button>
      </header>
      <section aria-label="项目列表" className="project-grid">
        <article className="project-card book-card">
          <div className="book-cover"><span>雾港</span><i>LETTERS</i></div>
          <div className="project-card-copy"><span className="project-type">书籍</span>
          <h2>《{demoBook.title}》</h2>
          <p>{demoBook.originalTitle}</p>
          <div className="card-progress"><span><i style={{ width: `${demoBook.progress}%` }} /></span><b>{demoBook.progress}%</b></div>
          <p className="project-meta">第八章 · 潮声背后 · 9 个待处理问题</p>
          <button type="button" className="card-action" aria-label={`打开《${demoBook.title}》`} onClick={onOpenBook}>继续校对 <span>→</span></button></div>
        </article>
        <article className="project-card game-card">
          <div className="game-art"><span>月</span><i /></div>
          <div className="project-card-copy"><span className="project-type">游戏</span>
            <h2>月光石物语</h2>
            <p>Moonstone Chronicle</p>
            <div className="card-progress"><span><i style={{ width: '42%' }} /></span><b>42%</b></div>
            <p className="project-meta">RPG Maker MZ · 翻译任务进行中</p>
            <button type="button" className="card-action secondary">查看任务 <span>→</span></button>
          </div>
        </article>
      </section>
    </main>
  )
}

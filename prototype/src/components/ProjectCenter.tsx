import { demoBook } from '../data/demoBook'

export function ProjectCenter({ onOpenBook }: { onOpenBook: () => void }) {
  return (
    <main>
      <p>PROJECT LIBRARY / 项目中心</p>
      <h1>所有项目</h1>
      <section aria-label="项目列表">
        <article>
          <span>书籍</span>
          <h2>《{demoBook.title}》</h2>
          <p>{demoBook.originalTitle}</p>
          <button type="button" aria-label={`打开《${demoBook.title}》`} onClick={onOpenBook}>继续校对</button>
        </article>
        <article>
          <span>游戏</span>
          <h2>月光石物语</h2>
          <p>RPG Maker MZ · 翻译任务进行中</p>
        </article>
      </section>
    </main>
  )
}

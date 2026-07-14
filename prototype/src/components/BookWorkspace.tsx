import { demoBook } from '../data/demoBook'

const navItems = ['书稿', '编辑', '问题', '术语', '导出']

export function BookWorkspace() {
  return (
    <main>
      <header>
        <p>BOOK / 书籍项目</p>
        <h1>{demoBook.title}</h1>
      </header>
      <nav aria-label="书籍项目导航">
        {navItems.map((item) => <button type="button" key={item}>{item}</button>)}
      </nav>
      <p>第八章 · 潮声背后</p>
    </main>
  )
}

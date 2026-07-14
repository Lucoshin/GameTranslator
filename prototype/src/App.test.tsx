import { fireEvent, render, screen } from '@testing-library/react'
import App from './App'

describe('项目中心', () => {
  it('展示项目导航和书籍项目类型', () => {
    render(<App />)

    expect(screen.getByRole('navigation', { name: '产品导航' })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: '所有项目' })).toBeInTheDocument()
    expect(screen.getByText('书籍')).toBeInTheDocument()
  })
})

describe('书籍工作台', () => {
  it('从项目中心进入书籍项目后展示专属导航', () => {
    render(<App />)

    fireEvent.click(screen.getByRole('button', { name: '打开《雾港来信》' }))

    const workspaceNav = screen.getByRole('navigation', { name: '书籍项目导航' })
    for (const label of ['书稿', '编辑', '问题', '术语', '导出']) {
      expect(workspaceNav).toHaveTextContent(label)
    }
  })
})

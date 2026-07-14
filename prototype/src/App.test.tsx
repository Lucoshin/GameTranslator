import { fireEvent, render, screen, waitFor } from '@testing-library/react'
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
  const openBook = () => {
    render(<App />)
    fireEvent.click(screen.getByRole('button', { name: '打开《雾港来信》' }))
  }

  it('从项目中心进入书籍项目后展示专属导航', () => {
    openBook()

    const workspaceNav = screen.getByRole('navigation', { name: '书籍项目导航' })
    for (const label of ['书稿', '编辑', '问题', '术语', '导出']) {
      expect(workspaceNav).toHaveTextContent(label)
    }
  })

  it('通过书籍深链接直接恢复工作台', () => {
    window.location.hash = '#book'

    render(<App />)

    expect(screen.getByRole('navigation', { name: '书籍项目导航' })).toBeInTheDocument()
    window.location.hash = ''
  })

  it('切换到逐段校对时保持当前段落', () => {
    openBook()

    fireEvent.click(screen.getByRole('button', { name: '选择段落 2' }))
    expect(screen.getByText('当前段落 2')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '逐段校对' }))

    expect(screen.getByRole('button', { name: '选择段落 2' })).toHaveAttribute('aria-current', 'true')
    expect(screen.getByText('当前段落 2')).toBeInTheDocument()
  })

  it('自动保存修改并用快捷键确认后前往下一段', async () => {
    openBook()

    const editor = screen.getByRole('textbox', { name: '段落 1 译文' })
    fireEvent.change(editor, { target: { value: '港口的雾比往年更早抵达。' } })
    expect(screen.getByText('正在保存…')).toBeInTheDocument()
    await waitFor(() => expect(screen.getByText('已保存')).toBeInTheDocument())

    fireEvent.keyDown(editor, { key: 'Enter', ctrlKey: true })

    expect(screen.getByText('段落 1 状态：已校对')).toBeInTheDocument()
    expect(screen.getByText('当前段落 2')).toBeInTheDocument()
  })
})

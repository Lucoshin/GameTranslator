import { render, screen } from '@testing-library/react'
import App from './App'

describe('项目中心', () => {
  it('展示项目导航和书籍项目类型', () => {
    render(<App />)

    expect(screen.getByRole('navigation', { name: '产品导航' })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: '所有项目' })).toBeInTheDocument()
    expect(screen.getByText('书籍')).toBeInTheDocument()
  })
})

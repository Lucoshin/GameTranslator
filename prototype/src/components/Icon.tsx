type IconName = 'projects' | 'book' | 'glossary' | 'settings' | 'history'

const paths: Record<IconName, string> = {
  projects: 'M4 5.5h6v5H4zM14 5.5h6v5h-6zM4 14.5h6v5H4zM14 14.5h6v5h-6z',
  book: 'M4.5 5.5A2.5 2.5 0 0 1 7 3h5v17H7a2.5 2.5 0 0 0-2.5 2.5zM19.5 5.5A2.5 2.5 0 0 0 17 3h-5v17h5a2.5 2.5 0 0 1 2.5 2.5z',
  glossary: 'M5 4h14M5 8h9M5 12h14M5 16h9M5 20h14',
  settings: 'M12 8.5a3.5 3.5 0 1 0 0 7 3.5 3.5 0 0 0 0-7zM19 12a7 7 0 0 0-.08-1l2-1.5-2-3.46-2.38.96a7.14 7.14 0 0 0-1.73-1L14.5 3h-4l-.31 3a7.14 7.14 0 0 0-1.73 1L6.08 6.04l-2 3.46 2 1.5A7 7 0 0 0 6 12c0 .34.03.67.08 1l-2 1.5 2 3.46L8.46 17a7.14 7.14 0 0 0 1.73 1l.31 3h4l.31-3a7.14 7.14 0 0 0 1.73-1l2.38.96 2-3.46-2-1.5c.05-.33.08-.66.08-1z',
  history: 'M4 12a8 8 0 1 0 2.34-5.66L4 8.68M4 4v4.68h4.68M12 7v5l3.5 2',
}

export function Icon({ name }: { name: IconName }) {
  return (
    <svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
      <path d={paths[name]} />
    </svg>
  )
}

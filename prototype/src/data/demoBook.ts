export const demoBook = {
  id: 'mist-harbor-letters',
  title: '雾港来信',
  originalTitle: 'Letters from the Mist Harbor',
  author: 'Eleanor Vale',
  progress: 68,
}

export type SegmentStatus = 'draft' | 'reviewed' | 'issue'

export type BookSegment = {
  id: number
  source: string
  translation: string
  status: SegmentStatus
  note?: string
  terms?: string[]
}

export const chapters = [
  { number: '01', title: '雾中的灯塔', progress: 100, issues: 0 },
  { number: '02', title: '未寄出的信', progress: 100, issues: 0 },
  { number: '03', title: '旧码头以北', progress: 92, issues: 1 },
  { number: '04', title: '守夜人的地图', progress: 84, issues: 2 },
  { number: '05', title: '盐与铁锈', progress: 76, issues: 0 },
  { number: '06', title: '候潮室', progress: 71, issues: 3 },
  { number: '07', title: '没有名字的船', progress: 69, issues: 1 },
  { number: '08', title: '潮声背后', progress: 63, issues: 2, active: true },
  { number: '09', title: '回信', progress: 28, issues: 0 },
]

export const initialSegments: BookSegment[] = [
  {
    id: 1,
    source: 'The fog reached the harbor earlier than it had in previous years.',
    translation: '港口的雾，比往年更早抵达。',
    status: 'draft',
    terms: ['harbor → 港口'],
  },
  {
    id: 2,
    source: 'By dusk, the lamps along Pilgrim Street had become pale islands in a white sea.',
    translation: '黄昏时，朝圣者街沿途的灯火，已成了白色海洋中一座座苍白的孤岛。',
    status: 'issue',
    note: '“pale”与“白色”语义重复，建议简化。',
    terms: ['Pilgrim Street → 朝圣者街'],
  },
  {
    id: 3,
    source: 'Mara stood at the attic window, holding the unopened letter against her palm.',
    translation: '玛拉站在阁楼窗前，把那封尚未拆开的信贴在掌心。',
    status: 'reviewed',
    terms: ['Mara → 玛拉'],
  },
  {
    id: 4,
    source: 'The paper was cold, though it had spent all afternoon beside the stove.',
    translation: '纸页仍是冰凉的，尽管它整个下午都放在炉火旁。',
    status: 'draft',
  },
  {
    id: 5,
    source: 'There was no return address, only the small ink mark she had learned to fear.',
    translation: '信封上没有回邮地址，只有那个她早已学会畏惧的小小墨记。',
    status: 'draft',
  },
]

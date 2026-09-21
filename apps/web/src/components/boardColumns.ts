import type { TaskStatus } from '../api/client'

export const BOARD_COLUMNS: { status: TaskStatus; label: string }[] = [
  { status: 'backlog', label: 'Backlog' },
  { status: 'doing', label: 'Doing' },
  { status: 'done', label: 'Done' },
]

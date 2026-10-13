import { useState, type FormEvent } from 'react'
import type { Task, TaskStatus } from '../api/client'
import { BOARD_COLUMNS } from './boardColumns'
import { ProjectCard } from './ProjectCard'

const COLUMN_ACCENT: Record<TaskStatus, string> = {
  backlog: 'var(--col-backlog)',
  doing: 'var(--col-doing)',
  done: 'var(--col-done)',
}

type ProjectBoardProps = {
  tasks: Task[]
  movingId: string | null
  moveError: string | null
  onMoveTask: (task: Task, status: TaskStatus) => void | Promise<void>
  onCreateTask: (title: string, status: TaskStatus) => Promise<boolean>
  createError: string | null
  creating: boolean
}

export function ProjectBoard({
  tasks,
  movingId,
  moveError,
  onMoveTask,
  onCreateTask,
  createError,
  creating,
}: ProjectBoardProps) {
  const [dragTaskId, setDragTaskId] = useState<string | null>(null)
  const [dropTarget, setDropTarget] = useState<TaskStatus | null>(null)
  const [quickTitle, setQuickTitle] = useState('')
  const [columnTitle, setColumnTitle] = useState('')
  const [columnAddOpen, setColumnAddOpen] = useState(false)

  async function submitQuickAdd(e: FormEvent) {
    e.preventDefault()
    const trimmed = quickTitle.trim()
    if (!trimmed) return
    const ok = await onCreateTask(trimmed, 'backlog')
    if (ok) setQuickTitle('')
  }

  async function submitColumnAdd(e: FormEvent) {
    e.preventDefault()
    const trimmed = columnTitle.trim()
    if (!trimmed) return
    const ok = await onCreateTask(trimmed, 'backlog')
    if (ok) {
      setColumnTitle('')
      setColumnAddOpen(false)
    }
  }

  function handleDrop(status: TaskStatus) {
    if (!dragTaskId) return
    const task = tasks.find((t) => t.id === dragTaskId)
    setDropTarget(null)
    setDragTaskId(null)
    if (!task || task.status === status) return
    void onMoveTask(task, status)
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-col gap-3 rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)]/90 px-4 py-3 shadow-sm sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h2 className="text-sm font-semibold tracking-wide text-[var(--ink)]">Board</h2>
          <p className="mt-0.5 text-xs text-[var(--muted)]">
            Drag cards across columns · keyboard Move on each card
          </p>
        </div>
        <form
          onSubmit={submitQuickAdd}
          className="flex w-full min-w-0 flex-col gap-2 sm:max-w-lg sm:flex-row sm:items-center"
          aria-label="Add task to backlog"
        >
          <label htmlFor="board-quick-add" className="sr-only">
            New task title
          </label>
          <input
            id="board-quick-add"
            type="text"
            value={quickTitle}
            required
            maxLength={500}
            disabled={creating}
            placeholder="Add a task to backlog…"
            onChange={(e) => setQuickTitle(e.target.value)}
            className="cc-input min-w-0 flex-1 disabled:opacity-60"
          />
          <button
            type="submit"
            disabled={creating}
            className="cc-btn-primary shrink-0 px-4 py-2"
          >
            {creating ? 'Adding…' : 'Add'}
          </button>
        </form>
      </div>

      {createError ? (
        <p className="text-sm text-[var(--danger)]" role="alert">
          {createError}
        </p>
      ) : null}
      {moveError ? (
        <p className="text-sm text-[var(--danger)]" role="alert">
          {moveError}
        </p>
      ) : null}

      <div
        className="rounded-2xl border border-[var(--border)] bg-[var(--board-canvas)] p-3 sm:p-4"
        style={{
          backgroundImage:
            'radial-gradient(circle at 1px 1px, var(--board-dot) 1px, transparent 0)',
          backgroundSize: '18px 18px',
        }}
      >
        <div
          className="flex gap-3 overflow-x-auto pb-1 pt-0.5"
          aria-label="Task board"
          role="region"
        >
          {BOARD_COLUMNS.map((column) => {
            const columnTasks = tasks.filter((t) => t.status === column.status)
            const isDropTarget = dropTarget === column.status
            const accent = COLUMN_ACCENT[column.status]

            return (
              <section
                key={column.status}
                aria-labelledby={`column-${column.status}`}
                onDragOver={(e) => {
                  e.preventDefault()
                  e.dataTransfer.dropEffect = 'move'
                  if (dropTarget !== column.status) setDropTarget(column.status)
                }}
                onDragLeave={(e) => {
                  const related = e.relatedTarget as Node | null
                  if (related && e.currentTarget.contains(related)) return
                  if (dropTarget === column.status) setDropTarget(null)
                }}
                onDrop={(e) => {
                  e.preventDefault()
                  handleDrop(column.status)
                }}
                className={[
                  'flex w-[min(100%,17.5rem)] shrink-0 flex-col overflow-hidden rounded-xl border bg-[var(--bg)]/95',
                  'min-h-[26rem] shadow-sm sm:w-80',
                  'transition-[border-color,box-shadow] duration-150',
                  isDropTarget
                    ? 'border-[var(--accent)] shadow-[inset_0_0_0_2px_var(--drop-ring)]'
                    : 'border-[var(--border)]',
                ].join(' ')}
              >
                <header
                  className="sticky top-0 z-10 flex items-center justify-between gap-2 border-b border-[var(--border)] bg-[var(--bg-elevated)]/95 px-3 py-2.5 backdrop-blur-sm"
                  style={{ boxShadow: `inset 3px 0 0 ${accent}` }}
                >
                  <h3
                    id={`column-${column.status}`}
                    className="text-sm font-semibold tracking-wide text-[var(--ink)]"
                  >
                    {column.label}
                  </h3>
                  <span
                    className="rounded-full bg-[var(--bg)] px-2 py-0.5 text-xs font-semibold tabular-nums text-[var(--muted)] ring-1 ring-[var(--border)]"
                    aria-label={`${columnTasks.length} tasks`}
                  >
                    {columnTasks.length}
                  </span>
                </header>

                <ul className="flex flex-1 list-none flex-col gap-2.5 overflow-y-auto p-2.5">
                  {columnTasks.length === 0 ? (
                    <li
                      className={[
                        'flex min-h-[7.5rem] flex-1 items-center justify-center rounded-lg border border-dashed px-3 py-8 text-center text-sm',
                        isDropTarget
                          ? 'border-[var(--accent)] bg-[var(--accent)]/5 font-medium text-[var(--accent)]'
                          : 'border-[var(--border)] text-[var(--muted)]',
                      ].join(' ')}
                    >
                      {isDropTarget ? 'Drop card here' : 'No cards'}
                    </li>
                  ) : (
                    columnTasks.map((task) => (
                      <ProjectCard
                        key={task.id}
                        task={task}
                        moving={movingId === task.id}
                        isDragging={dragTaskId === task.id}
                        onMove={(next) => void onMoveTask(task, next)}
                        onDragStart={setDragTaskId}
                        onDragEnd={() => {
                          setDragTaskId(null)
                          setDropTarget(null)
                        }}
                      />
                    ))
                  )}
                </ul>

                {column.status === 'backlog' ? (
                  <div className="border-t border-[var(--border)] bg-[var(--bg-elevated)]/80 p-2.5">
                    {columnAddOpen ? (
                      <form
                        onSubmit={submitColumnAdd}
                        className="space-y-2"
                        aria-label="Add to backlog"
                      >
                        <label htmlFor="column-quick-add" className="sr-only">
                          Task title
                        </label>
                        <input
                          id="column-quick-add"
                          type="text"
                          value={columnTitle}
                          required
                          maxLength={500}
                          autoFocus
                          disabled={creating}
                          placeholder="Card title…"
                          onChange={(e) => setColumnTitle(e.target.value)}
                          onKeyDown={(e) => {
                            if (e.key === 'Escape') {
                              setColumnAddOpen(false)
                              setColumnTitle('')
                            }
                          }}
                          className="cc-input-sm disabled:opacity-60"
                        />
                        <div className="flex gap-2">
                          <button
                            type="submit"
                            disabled={creating}
                            className="cc-btn-primary disabled:opacity-60"
                          >
                            {creating ? 'Adding…' : 'Add card'}
                          </button>
                          <button
                            type="button"
                            onClick={() => {
                              setColumnAddOpen(false)
                              setColumnTitle('')
                            }}
                            className="rounded-md px-3 py-1.5 text-sm text-[var(--muted)] hover:text-[var(--ink)]"
                          >
                            Cancel
                          </button>
                        </div>
                      </form>
                    ) : (
                      <button
                        type="button"
                        onClick={() => setColumnAddOpen(true)}
                        className="w-full rounded-md px-2 py-2 text-left text-sm font-medium text-[var(--muted)] transition-colors hover:bg-[var(--bg)] hover:text-[var(--ink)]"
                      >
                        + Add a card
                      </button>
                    )}
                  </div>
                ) : null}
              </section>
            )
          })}
        </div>
      </div>
    </div>
  )
}

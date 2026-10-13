import type { Task, TaskStatus } from '../api/client'
import { BOARD_COLUMNS } from './boardColumns'

function formatDueOn(dueOn: string): string {
  const parsed = new Date(`${dueOn}T12:00:00`)
  if (Number.isNaN(parsed.getTime())) return dueOn
  return parsed.toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
  })
}

function isOverdue(dueOn: string): boolean {
  const today = new Date().toISOString().slice(0, 10)
  return dueOn < today
}

type ProjectCardProps = {
  task: Task
  moving: boolean
  onMove: (status: TaskStatus) => void
  onDragStart: (taskId: string) => void
  onDragEnd: () => void
  isDragging: boolean
}

export function ProjectCard({
  task,
  moving,
  onMove,
  onDragStart,
  onDragEnd,
  isDragging,
}: ProjectCardProps) {
  const status = task.status as TaskStatus
  const overdue = task.due_on ? isOverdue(task.due_on) : false

  return (
    <li
      draggable={!moving}
      onDragStart={(e) => {
        e.dataTransfer.setData('text/plain', task.id)
        e.dataTransfer.effectAllowed = 'move'
        onDragStart(task.id)
      }}
      onDragEnd={onDragEnd}
      className={[
        'group list-none rounded-lg border border-[var(--border)] bg-[var(--bg-elevated)] p-3',
        'shadow-[var(--shadow-card)] transition-[box-shadow,opacity,transform,border-color] duration-150',
        moving ? 'opacity-55' : 'hover:border-[var(--border)] hover:shadow-[var(--shadow-card-hover)]',
        isDragging ? 'scale-[1.02] opacity-45 ring-2 ring-[var(--accent)]/35' : '',
        !moving ? 'cursor-grab active:cursor-grabbing' : '',
      ]
        .filter(Boolean)
        .join(' ')}
    >
      <div className="flex items-start gap-2.5">
        <span
          className="mt-0.5 flex w-3 shrink-0 flex-col items-center gap-0.5 opacity-40 group-hover:opacity-80"
          aria-hidden
          title="Drag"
        >
          <span className="h-0.5 w-2.5 rounded-full bg-[var(--muted)]" />
          <span className="h-0.5 w-2.5 rounded-full bg-[var(--muted)]" />
          <span className="h-0.5 w-2.5 rounded-full bg-[var(--muted)]" />
        </span>
        <div className="min-w-0 flex-1">
          <p className="text-[0.9375rem] font-semibold leading-snug tracking-tight text-[var(--ink)]">
            {task.title}
          </p>
          {task.description ? (
            <p className="mt-1.5 line-clamp-2 text-sm leading-relaxed text-[var(--muted)]">
              {task.description}
            </p>
          ) : null}
          <div className="mt-2.5 flex flex-wrap items-center gap-2">
            {task.due_on ? (
              <span
                className={[
                  'inline-flex items-center rounded-md px-1.5 py-0.5 text-[0.6875rem] font-medium tabular-nums ring-1',
                  overdue
                    ? 'bg-red-50 text-[var(--danger)] ring-red-200'
                    : 'bg-[var(--bg)] text-[var(--muted)] ring-[var(--border)]',
                ].join(' ')}
              >
                {overdue ? 'Overdue · ' : ''}
                {formatDueOn(task.due_on)}
              </span>
            ) : null}
            <label htmlFor={`task-status-${task.id}`} className="sr-only">
              Move {task.title}
            </label>
            <select
              id={`task-status-${task.id}`}
              value={BOARD_COLUMNS.some((c) => c.status === status) ? status : 'backlog'}
              disabled={moving}
              onChange={(e) => onMove(e.target.value as TaskStatus)}
              onClick={(e) => e.stopPropagation()}
              onMouseDown={(e) => e.stopPropagation()}
              className="cc-input-sm ml-auto max-w-[7.5rem] py-1 text-[0.6875rem] font-medium text-[var(--muted)] focus-visible:ring-[var(--accent)]/25 disabled:opacity-60"
            >
              {BOARD_COLUMNS.map((opt) => (
                <option key={opt.status} value={opt.status}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>
    </li>
  )
}

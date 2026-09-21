import { useEffect, useMemo, useState } from 'react'
import { Link, useLocation, useParams } from 'react-router-dom'
import { api, type Project, type Task, type TaskStatus } from '../api/client'
import { ProjectBoard } from '../components/ProjectBoard'

export function ProjectDetailPage() {
  const { id } = useParams<{ id: string }>()
  const location = useLocation()
  const [project, setProject] = useState<Project | null>(null)
  const [tasks, setTasks] = useState<Task[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [creating, setCreating] = useState(false)
  const [createError, setCreateError] = useState<string | null>(null)
  const [moveError, setMoveError] = useState<string | null>(null)
  const [movingId, setMovingId] = useState<string | null>(null)

  async function load(projectId: string) {
    setLoading(true)
    setError(null)
    const [projectResult, tasksResult] = await Promise.all([
      api.getProject(projectId),
      api.listTasks(projectId),
    ])
    setLoading(false)
    if (!projectResult.ok) {
      setError(
        projectResult.status === 404
          ? 'Project not found.'
          : `Couldn’t open project — ${projectResult.error}`,
      )
      setProject(null)
      setTasks([])
      return
    }
    if (!tasksResult.ok) {
      setError(`Couldn’t load tasks — ${tasksResult.error}`)
      setProject(projectResult.data)
      setTasks([])
      return
    }
    setProject(projectResult.data)
    setTasks(tasksResult.data)
  }

  useEffect(() => {
    if (!id) {
      setLoading(false)
      setError('Missing project id.')
      return
    }
    let cancelled = false
    ;(async () => {
      setLoading(true)
      setError(null)
      const [projectResult, tasksResult] = await Promise.all([
        api.getProject(id),
        api.listTasks(id),
      ])
      if (cancelled) return
      setLoading(false)
      if (!projectResult.ok) {
        setError(
          projectResult.status === 404
            ? 'Project not found.'
            : `Couldn’t open project — ${projectResult.error}`,
        )
        setProject(null)
        setTasks([])
        return
      }
      if (!tasksResult.ok) {
        setError(`Couldn’t load tasks — ${tasksResult.error}`)
        setProject(projectResult.data)
        setTasks([])
        return
      }
      setProject(projectResult.data)
      setTasks(tasksResult.data)
    })()
    return () => {
      cancelled = true
    }
  }, [id, location.key, location.state])

  const counts = useMemo(() => {
    return {
      total: tasks.length,
      backlog: tasks.filter((t) => t.status === 'backlog').length,
      doing: tasks.filter((t) => t.status === 'doing').length,
      done: tasks.filter((t) => t.status === 'done').length,
    }
  }, [tasks])

  async function onCreateTask(title: string, status: TaskStatus): Promise<boolean> {
    if (!id) return false
    setCreateError(null)
    const trimmed = title.trim()
    if (!trimmed) {
      setCreateError('Title is required.')
      return false
    }
    setCreating(true)
    const result = await api.createTask(id, { title: trimmed, status })
    setCreating(false)
    if (!result.ok) {
      setCreateError(result.error)
      return false
    }
    setTasks((prev) => [...prev.filter((t) => t.id !== result.data.id), result.data])
    return true
  }

  async function moveTask(task: Task, status: TaskStatus) {
    if (task.status === status) return
    setMoveError(null)
    setMovingId(task.id)
    const previous = task
    setTasks((prev) => prev.map((t) => (t.id === task.id ? { ...t, status } : t)))
    const result = await api.patchTask(task.id, { status })
    setMovingId(null)
    if (!result.ok) {
      setTasks((prev) => prev.map((t) => (t.id === previous.id ? previous : t)))
      setMoveError(`Couldn’t move task — ${result.error}`)
      return
    }
    setTasks((prev) => prev.map((t) => (t.id === result.data.id ? result.data : t)))
  }

  return (
    <section className="space-y-5">
      <nav className="text-sm text-[var(--muted)]" aria-label="Breadcrumb">
        <Link
          to="/projects"
          className="underline-offset-2 hover:text-[var(--accent)] hover:underline"
        >
          Projects
        </Link>
        <span className="mx-2 text-[var(--border)]" aria-hidden>
          /
        </span>
        <span className="text-[var(--ink)]">{project?.name ?? 'Board'}</span>
      </nav>

      {loading ? (
        <p className="text-sm text-[var(--muted)]">Loading board…</p>
      ) : error && !project ? (
        <p className="text-sm text-[var(--danger)]" role="alert">
          {error}
        </p>
      ) : project ? (
        <>
          <header className="flex flex-col gap-3 border-b border-[var(--border)] pb-5 sm:flex-row sm:items-end sm:justify-between">
            <div className="min-w-0">
              <h1 className="truncate text-3xl font-semibold tracking-tight">{project.name}</h1>
              {project.description ? (
                <p className="mt-2 max-w-3xl whitespace-pre-wrap text-sm leading-relaxed text-[var(--muted)]">
                  {project.description}
                </p>
              ) : null}
            </div>
            <dl className="flex flex-wrap gap-2 text-xs font-medium">
              <div className="rounded-full bg-[var(--bg-elevated)] px-2.5 py-1 ring-1 ring-[var(--border)]">
                <dt className="sr-only">Total</dt>
                <dd className="tabular-nums text-[var(--muted)]">{counts.total} cards</dd>
              </div>
              <div className="rounded-full bg-[var(--bg-elevated)] px-2.5 py-1 ring-1 ring-[var(--border)]">
                <dt className="sr-only">Doing</dt>
                <dd className="tabular-nums text-[var(--accent)]">{counts.doing} doing</dd>
              </div>
              <div className="rounded-full bg-[var(--bg-elevated)] px-2.5 py-1 ring-1 ring-[var(--border)]">
                <dt className="sr-only">Done</dt>
                <dd className="tabular-nums text-[var(--col-done)]">{counts.done} done</dd>
              </div>
            </dl>
          </header>

          {error ? (
            <div className="space-y-2">
              <p className="text-sm text-[var(--danger)]" role="alert">
                {error}
              </p>
              <button
                type="button"
                onClick={() => id && void load(id)}
                className="text-sm text-[var(--accent)] underline-offset-2 hover:underline"
              >
                Retry
              </button>
            </div>
          ) : (
            <ProjectBoard
              tasks={tasks}
              movingId={movingId}
              moveError={moveError}
              onMoveTask={moveTask}
              onCreateTask={onCreateTask}
              createError={createError}
              creating={creating}
            />
          )}
        </>
      ) : null}
    </section>
  )
}

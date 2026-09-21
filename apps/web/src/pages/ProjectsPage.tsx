import { useEffect, useState, type FormEvent } from 'react'
import { Link } from 'react-router-dom'
import { api, type Project } from '../api/client'

export function ProjectsPage() {
  const [projects, setProjects] = useState<Project[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [formError, setFormError] = useState<string | null>(null)

  async function load() {
    setLoading(true)
    setError(null)
    const result = await api.listProjects()
    setLoading(false)
    if (!result.ok) {
      setError(result.error)
      return
    }
    setProjects(result.data)
  }

  useEffect(() => {
    void load()
  }, [])

  async function onCreate(e: FormEvent) {
    e.preventDefault()
    setFormError(null)
    const trimmed = name.trim()
    if (!trimmed) {
      setFormError('Name is required.')
      return
    }
    setSubmitting(true)
    const result = await api.createProject({
      name: trimmed,
      description: description.trim() || undefined,
    })
    setSubmitting(false)
    if (!result.ok) {
      setFormError(result.error)
      return
    }
    setName('')
    setDescription('')
    setProjects((prev) => [result.data, ...prev.filter((p) => p.id !== result.data.id)])
  }

  return (
    <section className="space-y-8">
      <div className="flex flex-col gap-1 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <h1 className="text-3xl font-semibold tracking-tight text-[var(--ink)]">Projects</h1>
          <p className="mt-2 max-w-xl text-[var(--muted)]">
            Your boards live here. Open one to plan work like Azure / GitHub Projects.
          </p>
        </div>
        <p className="text-sm tabular-nums text-[var(--muted)]">
          {loading ? '…' : `${projects.length} project${projects.length === 1 ? '' : 's'}`}
        </p>
      </div>

      <div className="grid gap-6 lg:grid-cols-[minmax(0,22rem)_1fr] lg:items-start">
        <form
          onSubmit={onCreate}
          className="space-y-4 rounded-2xl border border-[var(--border)] bg-[var(--bg-elevated)] p-5 shadow-[var(--shadow-card)]"
          aria-label="Create project"
        >
          <div>
            <h2 className="text-sm font-semibold tracking-wide text-[var(--ink)]">New project</h2>
            <p className="mt-1 text-xs text-[var(--muted)]">Creates an empty board you can fill.</p>
          </div>
          <div>
            <label htmlFor="project-name" className="mb-1 block text-sm font-medium">
              Name
            </label>
            <input
              id="project-name"
              type="text"
              value={name}
              required
              maxLength={200}
              disabled={submitting}
              placeholder="e.g. Side project, Home reno"
              onChange={(e) => setName(e.target.value)}
              className="w-full rounded-md border border-[var(--border)] bg-white px-3 py-2 text-sm outline-none focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/20"
            />
          </div>
          <div>
            <label htmlFor="project-description" className="mb-1 block text-sm font-medium">
              Description <span className="font-normal text-[var(--muted)]">(optional)</span>
            </label>
            <textarea
              id="project-description"
              value={description}
              maxLength={2000}
              rows={3}
              disabled={submitting}
              onChange={(e) => setDescription(e.target.value)}
              className="w-full rounded-md border border-[var(--border)] bg-white px-3 py-2 text-sm outline-none focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/20"
            />
          </div>
          {formError ? (
            <p className="text-sm text-[var(--danger)]" role="alert">
              {formError}
            </p>
          ) : null}
          <button
            type="submit"
            disabled={submitting}
            className="w-full rounded-md bg-[var(--accent)] px-4 py-2.5 text-sm font-medium text-white hover:bg-[var(--accent-hover)] disabled:opacity-60 sm:w-auto"
          >
            {submitting ? 'Creating…' : 'Create project'}
          </button>
        </form>

        <div>
          <h2 className="mb-3 text-sm font-semibold tracking-wide text-[var(--muted)]">
            Your boards
          </h2>
          {loading ? (
            <div className="grid gap-3 sm:grid-cols-2">
              {[0, 1].map((i) => (
                <div
                  key={i}
                  className="h-28 animate-pulse rounded-2xl border border-[var(--border)] bg-[var(--bg-elevated)]/70"
                />
              ))}
            </div>
          ) : error ? (
            <div className="space-y-2 rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4">
              <p className="text-sm text-[var(--danger)]" role="alert">
                Couldn’t load projects — {error}
              </p>
              <button
                type="button"
                onClick={() => void load()}
                className="text-sm text-[var(--accent)] underline-offset-2 hover:underline"
              >
                Retry
              </button>
            </div>
          ) : projects.length === 0 ? (
            <div className="rounded-2xl border border-dashed border-[var(--border)] bg-[var(--bg-elevated)]/60 px-5 py-10 text-center">
              <p className="font-medium text-[var(--ink)]">No boards yet</p>
              <p className="mt-1 text-sm text-[var(--muted)]">
                Create a project on the left to open your first Kanban board.
              </p>
            </div>
          ) : (
            <ul className="grid gap-3 sm:grid-cols-2">
              {projects.map((project) => (
                <li key={project.id}>
                  <Link
                    to={`/projects/${project.id}`}
                    className={[
                      'group flex h-full flex-col rounded-2xl border border-[var(--border)] bg-[var(--bg-elevated)] p-4',
                      'shadow-[var(--shadow-card)] transition-[box-shadow,border-color,transform] duration-150',
                      'hover:-translate-y-0.5 hover:border-[var(--accent)]/35 hover:shadow-[var(--shadow-card-hover)]',
                      'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/35',
                    ].join(' ')}
                  >
                    <span className="text-base font-semibold tracking-tight text-[var(--ink)]">
                      {project.name}
                    </span>
                    {project.description ? (
                      <span className="mt-1.5 line-clamp-2 flex-1 text-sm leading-relaxed text-[var(--muted)]">
                        {project.description}
                      </span>
                    ) : (
                      <span className="mt-1.5 flex-1 text-sm text-[var(--muted)]">No description</span>
                    )}
                    <span className="mt-4 inline-flex items-center gap-1 text-sm font-semibold text-[var(--accent)]">
                      Open board
                      <span aria-hidden className="transition-transform group-hover:translate-x-0.5">
                        →
                      </span>
                    </span>
                  </Link>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </section>
  )
}

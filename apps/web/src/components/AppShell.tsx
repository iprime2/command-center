import { useEffect, useState, type FormEvent } from 'react'
import { NavLink, Outlet, useMatch, useNavigate } from 'react-router-dom'
import { api } from '../api/client'
import { useAuth } from '../auth/AuthContext'
import { ThemeToggle } from './ThemeToggle'

const navItems = [
  { to: '/', label: 'Today', end: true },
  { to: '/projects', label: 'Projects' },
  { to: '/money', label: 'Money' },
  { to: '/body', label: 'Body' },
  { to: '/ai', label: 'AI' },
  { to: '/review', label: 'Review' },
] as const

export function AppShell() {
  const { user, logout } = useAuth()
  const navigate = useNavigate()
  const isProjectBoard = Boolean(useMatch('/projects/:id'))
  const mainMaxWidth = isProjectBoard ? 'max-w-[90rem]' : 'max-w-5xl'
  const headerMaxWidth = isProjectBoard ? 'max-w-[90rem]' : 'max-w-5xl'
  const [title, setTitle] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState<string | null>(null)

  useEffect(() => {
    if (!success) return
    const timer = window.setTimeout(() => setSuccess(null), 2500)
    return () => window.clearTimeout(timer)
  }, [success])

  async function onCapture(e: FormEvent) {
    e.preventDefault()
    setError(null)
    setSuccess(null)
    const trimmed = title.trim()
    if (!trimmed) {
      setError('Enter something to capture.')
      return
    }
    setSubmitting(true)
    const result = await api.capture({ title: trimmed })
    setSubmitting(false)
    if (!result.ok) {
      setError(result.error)
      return
    }
    setTitle('')
    setSuccess('Captured.')
    navigate(`/projects/${result.data.project_id}`, {
      state: { capturedTaskId: result.data.id },
    })
  }

  return (
    <div className="min-h-full flex flex-col">
      <header className="border-b border-[var(--border)] bg-[var(--bg-elevated)]/90 backdrop-blur-sm">
        <div className={`mx-auto flex ${headerMaxWidth} flex-wrap items-center justify-between gap-3 px-4 py-3`}>
          <div className="flex items-baseline gap-3">
            <span className="text-lg font-semibold tracking-tight text-[var(--accent)]">
              Command Center
            </span>
            <span className="hidden text-sm text-[var(--muted)] sm:inline">
              {user?.email}
            </span>
          </div>
          <form
            onSubmit={onCapture}
            className="order-last flex w-full min-w-0 flex-1 items-center gap-2 sm:order-none sm:max-w-md"
            aria-label="Quick capture"
          >
            <label htmlFor="quick-capture" className="sr-only">
              Quick capture
            </label>
            <input
              id="quick-capture"
              type="text"
              value={title}
              maxLength={500}
              disabled={submitting}
              placeholder="Capture a task…"
              onChange={(e) => setTitle(e.target.value)}
              className="cc-input-sm min-w-0 flex-1"
            />
            <button
              type="submit"
              disabled={submitting}
              className="cc-btn-primary shrink-0"
            >
              {submitting ? 'Saving…' : 'Capture'}
            </button>
          </form>
          <div className="flex items-center gap-2">
            <ThemeToggle />
            <button
              type="button"
              onClick={() => void logout()}
              className="rounded-md border border-[var(--border)] px-3 py-1.5 text-sm text-[var(--muted)] hover:border-[var(--accent)] hover:text-[var(--accent)]"
            >
              Log out
            </button>
          </div>
        </div>
        {(error || success) && (
          <div className={`mx-auto ${headerMaxWidth} px-4 pb-2`}>
            {error ? (
              <p className="text-sm text-[var(--danger)]" role="alert">
                Couldn’t capture — {error}
              </p>
            ) : null}
            {success ? (
              <p className="text-sm text-[var(--accent)]" role="status">
                {success}
              </p>
            ) : null}
          </div>
        )}
        <nav className={`mx-auto flex ${headerMaxWidth} gap-1 overflow-x-auto px-4 pb-2`} aria-label="Main">
          {navItems.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={'end' in item ? item.end : false}
              className={({ isActive }) =>
                [
                  'whitespace-nowrap rounded-md px-3 py-1.5 text-sm transition-colors',
                  isActive
                    ? 'bg-[var(--accent)] text-[var(--on-accent)]'
                    : 'text-[var(--muted)] hover:bg-[var(--bg)] hover:text-[var(--ink)]',
                ].join(' ')
              }
            >
              {item.label}
            </NavLink>
          ))}
        </nav>
      </header>
      <main className={`mx-auto w-full ${mainMaxWidth} flex-1 px-4 py-8`}>
        <Outlet />
      </main>
    </div>
  )
}

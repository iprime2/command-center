import { useState, type FormEvent, type ReactNode } from 'react'
import { Link, Navigate, useNavigate } from 'react-router-dom'
import { api } from '../api/client'
import { ThemeToggle } from '../components/ThemeToggle'
import { useAuth } from '../auth/AuthContext'

export function LoginPage() {
  const { user, loading, setUser } = useAuth()
  const navigate = useNavigate()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  if (!loading && user) return <Navigate to="/" replace />

  async function onSubmit(e: FormEvent) {
    e.preventDefault()
    setError(null)
    setSubmitting(true)
    const result = await api.login(email, password)
    setSubmitting(false)
    if (!result.ok) {
      setError(result.error)
      return
    }
    setUser(result.data.user)
    navigate('/', { replace: true })
  }

  return (
    <AuthLayout title="Log in" subtitle="Sign in to your Command Center.">
      <form onSubmit={onSubmit} className="space-y-4">
        <Field
          id="email"
          label="Email"
          type="email"
          autoComplete="email"
          value={email}
          onChange={setEmail}
          required
        />
        <Field
          id="password"
          label="Password"
          type="password"
          autoComplete="current-password"
          value={password}
          onChange={setPassword}
          required
          minLength={8}
        />
        {error ? (
          <p className="text-sm text-[var(--danger)]" role="alert">
            {error}
          </p>
        ) : null}
        <button
          type="submit"
          disabled={submitting}
          className="cc-btn-primary w-full px-4 py-2.5"
        >
          {submitting ? 'Signing in…' : 'Log in'}
        </button>
      </form>
      <p className="mt-6 text-sm text-[var(--muted)]">
        No account?{' '}
        <Link className="text-[var(--accent)] underline-offset-2 hover:underline" to="/register">
          Register
        </Link>
      </p>
    </AuthLayout>
  )
}

export function RegisterPage() {
  const { user, loading, setUser } = useAuth()
  const navigate = useNavigate()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  if (!loading && user) return <Navigate to="/" replace />

  async function onSubmit(e: FormEvent) {
    e.preventDefault()
    setError(null)
    setSubmitting(true)
    const result = await api.register(email, password)
    setSubmitting(false)
    if (!result.ok) {
      setError(result.error)
      return
    }
    setUser(result.data.user)
    navigate('/', { replace: true })
  }

  return (
    <AuthLayout title="Create account" subtitle="Personal use — email and password.">
      <form onSubmit={onSubmit} className="space-y-4">
        <Field
          id="email"
          label="Email"
          type="email"
          autoComplete="email"
          value={email}
          onChange={setEmail}
          required
        />
        <Field
          id="password"
          label="Password (min 8)"
          type="password"
          autoComplete="new-password"
          value={password}
          onChange={setPassword}
          required
          minLength={8}
        />
        {error ? (
          <p className="text-sm text-[var(--danger)]" role="alert">
            {error}
          </p>
        ) : null}
        <button
          type="submit"
          disabled={submitting}
          className="cc-btn-primary w-full px-4 py-2.5"
        >
          {submitting ? 'Creating…' : 'Register'}
        </button>
      </form>
      <p className="mt-6 text-sm text-[var(--muted)]">
        Already registered?{' '}
        <Link className="text-[var(--accent)] underline-offset-2 hover:underline" to="/login">
          Log in
        </Link>
      </p>
    </AuthLayout>
  )
}

function AuthLayout({
  title,
  subtitle,
  children,
}: {
  title: string
  subtitle: string
  children: ReactNode
}) {
  return (
    <div className="relative flex min-h-full items-center justify-center px-4 py-12">
      <div className="absolute right-4 top-4">
        <ThemeToggle />
      </div>
      <div className="w-full max-w-md rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] p-8 shadow-[var(--shadow-card)]">
        <p className="text-sm font-semibold tracking-wide text-[var(--accent)]">Command Center</p>
        <h1 className="mt-2 text-2xl font-semibold tracking-tight">{title}</h1>
        <p className="mt-1 text-sm text-[var(--muted)]">{subtitle}</p>
        <div className="mt-6">{children}</div>
      </div>
    </div>
  )
}

function Field({
  id,
  label,
  type,
  value,
  onChange,
  autoComplete,
  required,
  minLength,
}: {
  id: string
  label: string
  type: string
  value: string
  onChange: (v: string) => void
  autoComplete?: string
  required?: boolean
  minLength?: number
}) {
  return (
    <div>
      <label htmlFor={id} className="mb-1 block text-sm font-medium">
        {label}
      </label>
      <input
        id={id}
        type={type}
        value={value}
        autoComplete={autoComplete}
        required={required}
        minLength={minLength}
        onChange={(e) => onChange(e.target.value)}
        className="cc-input"
      />
    </div>
  )
}

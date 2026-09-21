import { useEffect, useState, type FormEvent } from 'react'
import { Link } from 'react-router-dom'
import {
  api,
  type BudgetSummary,
  type Expense,
  type ExpenseCategory,
  type TodayTask,
} from '../api/client'

export function TodayPage() {
  const [tasks, setTasks] = useState<TodayTask[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  async function load() {
    setLoading(true)
    setError(null)
    const result = await api.listToday()
    setLoading(false)
    if (!result.ok) {
      setError(result.error)
      return
    }
    setTasks(result.data)
  }

  useEffect(() => {
    void load()
  }, [])

  return (
    <section className="space-y-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Today</h1>
        <p className="mt-2 max-w-xl text-[var(--muted)]">
          In-progress work and anything due on or before today.
        </p>
      </div>

      {loading ? (
        <p className="text-sm text-[var(--muted)]">Loading today…</p>
      ) : error ? (
        <div className="space-y-2">
          <p className="text-sm text-[var(--danger)]" role="alert">
            Couldn’t load today — {error}
          </p>
          <button
            type="button"
            onClick={() => void load()}
            className="text-sm text-[var(--accent)] underline-offset-2 hover:underline"
          >
            Retry
          </button>
        </div>
      ) : tasks.length === 0 ? (
        <div className="space-y-2 border-t border-[var(--border)] pt-6">
          <p className="text-[var(--muted)]">Nothing on your plate for today.</p>
          <p className="text-sm text-[var(--muted)]">
            Move a task to Doing or set a due date on a{' '}
            <Link
              to="/projects"
              className="text-[var(--accent)] underline-offset-2 hover:underline"
            >
              project board
            </Link>
            .
          </p>
        </div>
      ) : (
        <ul className="divide-y divide-[var(--border)] border-t border-[var(--border)]">
          {tasks.map((task) => (
            <li key={task.id}>
              <Link
                to={`/projects/${task.project_id}`}
                className="flex flex-col gap-0.5 py-3 hover:text-[var(--accent)]"
              >
                <span className="font-medium">{task.title}</span>
                <span className="text-sm text-[var(--muted)]">
                  {task.project_name}
                  <span className="mx-1.5 text-[var(--border)]" aria-hidden>
                    ·
                  </span>
                  {statusLabel(task.status)}
                  {task.due_on ? (
                    <>
                      <span className="mx-1.5 text-[var(--border)]" aria-hidden>
                        ·
                      </span>
                      Due {task.due_on}
                    </>
                  ) : null}
                </span>
              </Link>
            </li>
          ))}
        </ul>
      )}
    </section>
  )
}

function statusLabel(status: string): string {
  switch (status) {
    case 'backlog':
      return 'Backlog'
    case 'doing':
      return 'Doing'
    case 'done':
      return 'Done'
    default:
      return status
  }
}

export function PlaceholderPage({ title }: { title: string }) {
  return (
    <section>
      <h1 className="text-2xl font-semibold tracking-tight">{title}</h1>
      <p className="mt-2 text-[var(--muted)]">Coming in a later slice.</p>
    </section>
  )
}

function formatInrFromPaise(paise: number): string {
  const rupees = paise / 100
  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
  }).format(rupees)
}

function utcTodayIso(): string {
  return new Date().toISOString().slice(0, 10)
}

function utcYearMonth(): { year: number; month: number } {
  const now = new Date()
  return { year: now.getUTCFullYear(), month: now.getUTCMonth() + 1 }
}

export function MoneyPage() {
  const { year, month } = utcYearMonth()
  const [categories, setCategories] = useState<ExpenseCategory[]>([])
  const [expenses, setExpenses] = useState<Expense[]>([])
  const [summary, setSummary] = useState<BudgetSummary | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const [amount, setAmount] = useState('')
  const [categoryId, setCategoryId] = useState('')
  const [spentOn, setSpentOn] = useState(utcTodayIso)
  const [note, setNote] = useState('')
  const [submitting, setSubmitting] = useState(false)
  const [formError, setFormError] = useState<string | null>(null)

  const [overallBudget, setOverallBudget] = useState('')
  const [categoryBudgetId, setCategoryBudgetId] = useState('')
  const [categoryBudgetAmount, setCategoryBudgetAmount] = useState('')
  const [budgetSubmitting, setBudgetSubmitting] = useState(false)
  const [budgetError, setBudgetError] = useState<string | null>(null)

  async function load() {
    setLoading(true)
    setError(null)
    const [catsResult, expensesResult, summaryResult] = await Promise.all([
      api.listExpenseCategories(),
      api.listExpenses(year, month),
      api.budgetSummary(year, month),
    ])
    setLoading(false)

    if (!catsResult.ok) {
      setError(catsResult.error)
      return
    }
    if (!expensesResult.ok) {
      setError(expensesResult.error)
      return
    }
    if (!summaryResult.ok) {
      setError(summaryResult.error)
      return
    }

    setCategories(catsResult.data)
    setExpenses(expensesResult.data)
    setSummary(summaryResult.data)
    if (summaryResult.data.overall_budget_paise != null) {
      setOverallBudget(String(summaryResult.data.overall_budget_paise / 100))
    }
    setCategoryId((prev) => {
      if (prev && catsResult.data.some((c) => c.id === prev)) return prev
      return catsResult.data[0]?.id ?? ''
    })
    setCategoryBudgetId((prev) => {
      if (prev && catsResult.data.some((c) => c.id === prev)) return prev
      return catsResult.data[0]?.id ?? ''
    })
  }

  useEffect(() => {
    void load()
  }, [])

  async function onCreate(e: FormEvent) {
    e.preventDefault()
    setFormError(null)

    const parsed = Number(amount)
    if (!Number.isFinite(parsed) || parsed <= 0) {
      setFormError('Enter a positive amount in rupees.')
      return
    }
    if (!categoryId) {
      setFormError('Choose a category.')
      return
    }
    if (!spentOn) {
      setFormError('Choose a date.')
      return
    }

    setSubmitting(true)
    const result = await api.createExpense({
      amount_rupees: parsed,
      category_id: categoryId,
      spent_on: spentOn,
      note: note.trim() || undefined,
    })
    setSubmitting(false)

    if (!result.ok) {
      setFormError(result.error)
      return
    }

    setAmount('')
    setNote('')
    setSpentOn(utcTodayIso())

    const inCurrentMonth =
      result.data.spent_on.startsWith(
        `${year}-${String(month).padStart(2, '0')}`,
      )
    if (inCurrentMonth) {
      setExpenses((prev) => [
        result.data,
        ...prev.filter((x) => x.id !== result.data.id),
      ])
      const summaryResult = await api.budgetSummary(year, month)
      if (summaryResult.ok) setSummary(summaryResult.data)
    }
  }

  async function onSaveOverallBudget(e: FormEvent) {
    e.preventDefault()
    setBudgetError(null)
    const parsed = Number(overallBudget)
    if (!Number.isFinite(parsed) || parsed <= 0) {
      setBudgetError('Enter a positive overall budget in rupees.')
      return
    }
    setBudgetSubmitting(true)
    const result = await api.upsertBudget({
      year,
      month,
      amount_rupees: parsed,
    })
    setBudgetSubmitting(false)
    if (!result.ok) {
      setBudgetError(result.error)
      return
    }
    const summaryResult = await api.budgetSummary(year, month)
    if (summaryResult.ok) setSummary(summaryResult.data)
  }

  async function onSaveCategoryBudget(e: FormEvent) {
    e.preventDefault()
    setBudgetError(null)
    const parsed = Number(categoryBudgetAmount)
    if (!Number.isFinite(parsed) || parsed <= 0) {
      setBudgetError('Enter a positive category budget in rupees.')
      return
    }
    if (!categoryBudgetId) {
      setBudgetError('Choose a category for the budget.')
      return
    }
    setBudgetSubmitting(true)
    const result = await api.upsertBudget({
      year,
      month,
      category_id: categoryBudgetId,
      amount_rupees: parsed,
    })
    setBudgetSubmitting(false)
    if (!result.ok) {
      setBudgetError(result.error)
      return
    }
    setCategoryBudgetAmount('')
    const summaryResult = await api.budgetSummary(year, month)
    if (summaryResult.ok) setSummary(summaryResult.data)
  }

  const categoryName = (id: string) =>
    categories.find((c) => c.id === id)?.name ?? 'Category'

  const monthLabel = new Intl.DateTimeFormat('en-IN', {
    month: 'long',
    year: 'numeric',
    timeZone: 'UTC',
  }).format(new Date(Date.UTC(year, month - 1, 1)))

  return (
    <section className="space-y-8">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Money</h1>
        <p className="mt-2 max-w-xl text-[var(--muted)]">
          Log INR expenses, set a monthly budget, and see spend vs remaining.
        </p>
      </div>

      <div className="border-t border-[var(--border)] pt-6">
        <h2 className="text-sm font-semibold tracking-wide text-[var(--muted)]">
          {monthLabel} — budget vs spend
        </h2>
        {loading ? (
          <p className="mt-3 text-sm text-[var(--muted)]">Loading budget…</p>
        ) : error ? (
          <div className="mt-3 space-y-2">
            <p className="text-sm text-[var(--danger)]" role="alert">
              Couldn’t load money data — {error}
            </p>
            <button
              type="button"
              onClick={() => void load()}
              className="text-sm text-[var(--accent)] underline-offset-2 hover:underline"
            >
              Retry
            </button>
          </div>
        ) : summary ? (
          <div className="mt-4 space-y-4">
            <dl className="grid max-w-lg gap-3 sm:grid-cols-3">
              <div>
                <dt className="text-xs font-medium uppercase tracking-wide text-[var(--muted)]">
                  Budget
                </dt>
                <dd className="mt-1 text-lg font-semibold tabular-nums">
                  {summary.overall_budget_paise != null
                    ? formatInrFromPaise(summary.overall_budget_paise)
                    : '—'}
                </dd>
              </div>
              <div>
                <dt className="text-xs font-medium uppercase tracking-wide text-[var(--muted)]">
                  Spent
                </dt>
                <dd className="mt-1 text-lg font-semibold tabular-nums">
                  {formatInrFromPaise(summary.total_spent_paise)}
                </dd>
              </div>
              <div>
                <dt className="text-xs font-medium uppercase tracking-wide text-[var(--muted)]">
                  Remaining
                </dt>
                <dd
                  className={`mt-1 text-lg font-semibold tabular-nums ${
                    summary.remaining_paise != null && summary.remaining_paise < 0
                      ? 'text-[var(--danger)]'
                      : ''
                  }`}
                >
                  {summary.remaining_paise != null
                    ? formatInrFromPaise(summary.remaining_paise)
                    : '—'}
                </dd>
              </div>
            </dl>

            {summary.by_category.length > 0 ? (
              <ul className="max-w-lg divide-y divide-[var(--border)] border-t border-[var(--border)]">
                {summary.by_category.map((row) => (
                  <li
                    key={row.category_id}
                    className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 py-2.5"
                  >
                    <span className="font-medium">{row.category_name}</span>
                    <span className="text-sm text-[var(--muted)] tabular-nums">
                      {formatInrFromPaise(row.spent_paise)} spent
                      {row.budget_paise != null ? (
                        <>
                          <span className="mx-1.5 text-[var(--border)]" aria-hidden>
                            ·
                          </span>
                          {row.remaining_paise != null
                            ? `${formatInrFromPaise(row.remaining_paise)} left`
                            : null}
                          <span className="mx-1.5 text-[var(--border)]" aria-hidden>
                            ·
                          </span>
                          of {formatInrFromPaise(row.budget_paise)}
                        </>
                      ) : null}
                    </span>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="text-sm text-[var(--muted)]">
                No category spend or budgets yet this month.
              </p>
            )}
          </div>
        ) : null}

        <form
          onSubmit={onSaveOverallBudget}
          className="mt-6 max-w-lg space-y-3"
          aria-label="Set overall monthly budget"
        >
          <h3 className="text-sm font-semibold tracking-wide text-[var(--muted)]">
            Overall monthly budget
          </h3>
          <div className="flex flex-wrap items-end gap-3">
            <div className="min-w-[10rem] flex-1">
              <label htmlFor="overall-budget" className="mb-1 block text-sm font-medium">
                Amount (₹)
              </label>
              <input
                id="overall-budget"
                type="number"
                inputMode="decimal"
                min="0.01"
                step="0.01"
                value={overallBudget}
                required
                disabled={budgetSubmitting || loading}
                onChange={(e) => setOverallBudget(e.target.value)}
                className="w-full rounded-md border border-[var(--border)] bg-white px-3 py-2 text-sm outline-none focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/20"
              />
            </div>
            <button
              type="submit"
              disabled={budgetSubmitting || loading}
              className="rounded-md bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--accent-hover)] disabled:opacity-60"
            >
              {budgetSubmitting ? 'Saving…' : 'Save budget'}
            </button>
          </div>
        </form>

        <form
          onSubmit={onSaveCategoryBudget}
          className="mt-4 max-w-lg space-y-3"
          aria-label="Set category monthly budget"
        >
          <h3 className="text-sm font-semibold tracking-wide text-[var(--muted)]">
            Category budget <span className="font-normal">(optional)</span>
          </h3>
          <div className="flex flex-wrap items-end gap-3">
            <div className="min-w-[8rem] flex-1">
              <label htmlFor="cat-budget-category" className="mb-1 block text-sm font-medium">
                Category
              </label>
              <select
                id="cat-budget-category"
                value={categoryBudgetId}
                required
                disabled={budgetSubmitting || loading || categories.length === 0}
                onChange={(e) => setCategoryBudgetId(e.target.value)}
                className="w-full rounded-md border border-[var(--border)] bg-white px-3 py-2 text-sm outline-none focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/20"
              >
                {categories.length === 0 ? (
                  <option value="">No categories</option>
                ) : (
                  categories.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name}
                    </option>
                  ))
                )}
              </select>
            </div>
            <div className="min-w-[8rem] flex-1">
              <label htmlFor="cat-budget-amount" className="mb-1 block text-sm font-medium">
                Amount (₹)
              </label>
              <input
                id="cat-budget-amount"
                type="number"
                inputMode="decimal"
                min="0.01"
                step="0.01"
                value={categoryBudgetAmount}
                required
                disabled={budgetSubmitting || loading}
                onChange={(e) => setCategoryBudgetAmount(e.target.value)}
                className="w-full rounded-md border border-[var(--border)] bg-white px-3 py-2 text-sm outline-none focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/20"
              />
            </div>
            <button
              type="submit"
              disabled={budgetSubmitting || loading}
              className="rounded-md border border-[var(--border)] px-4 py-2 text-sm font-medium hover:border-[var(--accent)] disabled:opacity-60"
            >
              {budgetSubmitting ? 'Saving…' : 'Save'}
            </button>
          </div>
          {budgetError ? (
            <p className="text-sm text-[var(--danger)]" role="alert">
              {budgetError}
            </p>
          ) : null}
        </form>
      </div>

      <form
        onSubmit={onCreate}
        className="max-w-lg space-y-3 border-t border-[var(--border)] pt-6"
        aria-label="Add expense"
      >
        <h2 className="text-sm font-semibold tracking-wide text-[var(--muted)]">
          Add expense
        </h2>
        <div>
          <label htmlFor="expense-amount" className="mb-1 block text-sm font-medium">
            Amount (₹)
          </label>
          <input
            id="expense-amount"
            type="number"
            inputMode="decimal"
            min="0.01"
            step="0.01"
            value={amount}
            required
            disabled={submitting || loading}
            onChange={(e) => setAmount(e.target.value)}
            className="w-full rounded-md border border-[var(--border)] bg-white px-3 py-2 text-sm outline-none focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/20"
          />
        </div>
        <div>
          <label htmlFor="expense-category" className="mb-1 block text-sm font-medium">
            Category
          </label>
          <select
            id="expense-category"
            value={categoryId}
            required
            disabled={submitting || loading || categories.length === 0}
            onChange={(e) => setCategoryId(e.target.value)}
            className="w-full rounded-md border border-[var(--border)] bg-white px-3 py-2 text-sm outline-none focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/20"
          >
            {categories.length === 0 ? (
              <option value="">No categories</option>
            ) : (
              categories.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name}
                </option>
              ))
            )}
          </select>
        </div>
        <div>
          <label htmlFor="expense-date" className="mb-1 block text-sm font-medium">
            Date
          </label>
          <input
            id="expense-date"
            type="date"
            value={spentOn}
            required
            disabled={submitting || loading}
            onChange={(e) => setSpentOn(e.target.value)}
            className="w-full rounded-md border border-[var(--border)] bg-white px-3 py-2 text-sm outline-none focus:border-[var(--accent)] focus:ring-2 focus:ring-[var(--accent)]/20"
          />
        </div>
        <div>
          <label htmlFor="expense-note" className="mb-1 block text-sm font-medium">
            Note <span className="font-normal text-[var(--muted)]">(optional)</span>
          </label>
          <input
            id="expense-note"
            type="text"
            value={note}
            maxLength={2000}
            disabled={submitting || loading}
            onChange={(e) => setNote(e.target.value)}
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
          disabled={submitting || loading}
          className="rounded-md bg-[var(--accent)] px-4 py-2 text-sm font-medium text-white hover:bg-[var(--accent-hover)] disabled:opacity-60"
        >
          {submitting ? 'Saving…' : 'Add expense'}
        </button>
      </form>

      <div className="border-t border-[var(--border)] pt-6">
        <h2 className="text-sm font-semibold tracking-wide text-[var(--muted)]">
          Expenses — {monthLabel}
        </h2>
        {loading ? (
          <p className="mt-3 text-sm text-[var(--muted)]">Loading expenses…</p>
        ) : error ? null : expenses.length === 0 ? (
          <p className="mt-3 text-sm text-[var(--muted)]">
            No expenses this month yet. Add one above.
          </p>
        ) : (
          <ul className="mt-3 divide-y divide-[var(--border)]">
            {expenses.map((expense) => (
              <li
                key={expense.id}
                className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 py-3"
              >
                <div className="min-w-0">
                  <p className="font-medium">{formatInrFromPaise(expense.amount_paise)}</p>
                  <p className="text-sm text-[var(--muted)]">
                    {categoryName(expense.category_id)}
                    <span className="mx-1.5 text-[var(--border)]" aria-hidden>
                      ·
                    </span>
                    {expense.spent_on}
                    {expense.note ? (
                      <>
                        <span className="mx-1.5 text-[var(--border)]" aria-hidden>
                          ·
                        </span>
                        {expense.note}
                      </>
                    ) : null}
                  </p>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </section>
  )
}

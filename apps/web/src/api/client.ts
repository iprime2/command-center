const API_URL = import.meta.env.VITE_API_URL ?? 'http://127.0.0.1:8080'

export type PublicUser = {
  id: string
  email: string
  created_at: string
  updated_at: string
}

export type AuthResponse = {
  user: PublicUser
}

export type Project = {
  id: string
  user_id: string
  name: string
  description: string | null
  status: string
  created_at: string
  updated_at: string
}

export type CreateProjectInput = {
  name: string
  description?: string
}

export type TaskStatus = 'backlog' | 'doing' | 'done'

export type Task = {
  id: string
  project_id: string
  user_id: string
  title: string
  description: string | null
  status: TaskStatus | string
  due_on: string | null
  position: number
  created_at: string
  updated_at: string
}

export type CreateTaskInput = {
  title: string
  description?: string
  status?: TaskStatus
  due_on?: string
}

export type CaptureInput = {
  title: string
  project_id?: string
}

export type PatchTaskInput = {
  title?: string
  description?: string
  status?: TaskStatus
  due_on?: string
}

export type TodayTask = {
  id: string
  project_id: string
  title: string
  status: TaskStatus | string
  due_on: string | null
  project_name: string
}

export type ExpenseCategory = {
  id: string
  user_id: string
  name: string
  created_at: string
  updated_at: string
}

export type Expense = {
  id: string
  user_id: string
  category_id: string
  amount_paise: number
  currency: string
  spent_on: string
  note: string | null
  project_id: string | null
  created_at: string
  updated_at: string
}

export type CreateExpenseInput = {
  amount_rupees: number
  category_id: string
  spent_on: string
  note?: string
  project_id?: string
}

export type Budget = {
  id: string
  user_id: string
  category_id: string | null
  year: number
  month: number
  amount_paise: number
  created_at: string
  updated_at: string
}

export type UpsertBudgetInput = {
  year: number
  month: number
  category_id?: string
  amount_rupees: number
}

export type CategoryBudgetSummary = {
  category_id: string
  category_name: string
  budget_paise: number | null
  spent_paise: number
  remaining_paise: number | null
}

export type BudgetSummary = {
  year: number
  month: number
  overall_budget_paise: number | null
  total_spent_paise: number
  remaining_paise: number | null
  by_category: CategoryBudgetSummary[]
}

export type ApiErrorBody = {
  error: {
    code: string
    message: string
  }
}

async function parseJson<T>(res: Response): Promise<T> {
  return (await res.json()) as T
}

async function request<T>(
  path: string,
  init: RequestInit = {},
): Promise<{ ok: true; data: T; status: number } | { ok: false; error: string; status: number }> {
  try {
    const res = await fetch(`${API_URL}${path}`, {
      ...init,
      credentials: 'include',
      headers: {
        Accept: 'application/json',
        ...(init.body ? { 'Content-Type': 'application/json' } : {}),
        ...init.headers,
      },
    })

    if (res.status === 204) {
      return { ok: true, data: undefined as T, status: res.status }
    }

    const data = await parseJson<T | ApiErrorBody>(res)
    if (!res.ok) {
      const message =
        data && typeof data === 'object' && 'error' in data
          ? (data as ApiErrorBody).error.message
          : `Request failed (${res.status})`
      return { ok: false, error: message, status: res.status }
    }
    return { ok: true, data: data as T, status: res.status }
  } catch {
    return { ok: false, error: 'Unable to reach API', status: 0 }
  }
}

export const api = {
  baseUrl: API_URL,
  me: () => request<PublicUser>('/auth/me'),
  register: (email: string, password: string) =>
    request<AuthResponse>('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    }),
  login: (email: string, password: string) =>
    request<AuthResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    }),
  logout: () => request<void>('/auth/logout', { method: 'POST' }),
  listProjects: () => request<Project[]>('/projects'),
  createProject: (input: CreateProjectInput) =>
    request<Project>('/projects', {
      method: 'POST',
      body: JSON.stringify(input),
    }),
  getProject: (id: string) => request<Project>(`/projects/${id}`),
  listTasks: (projectId: string) => request<Task[]>(`/projects/${projectId}/tasks`),
  createTask: (projectId: string, input: CreateTaskInput) =>
    request<Task>(`/projects/${projectId}/tasks`, {
      method: 'POST',
      body: JSON.stringify(input),
    }),
  capture: (input: CaptureInput) =>
    request<Task>('/capture', {
      method: 'POST',
      body: JSON.stringify(input),
    }),
  patchTask: (taskId: string, input: PatchTaskInput) =>
    request<Task>(`/tasks/${taskId}`, {
      method: 'PATCH',
      body: JSON.stringify(input),
    }),
  listToday: () => request<TodayTask[]>('/today'),
  listExpenseCategories: () => request<ExpenseCategory[]>('/expense-categories'),
  listExpenses: (year?: number, month?: number) => {
    const params = new URLSearchParams()
    if (year != null) params.set('year', String(year))
    if (month != null) params.set('month', String(month))
    const qs = params.toString()
    return request<Expense[]>(`/expenses${qs ? `?${qs}` : ''}`)
  },
  createExpense: (input: CreateExpenseInput) =>
    request<Expense>('/expenses', {
      method: 'POST',
      body: JSON.stringify(input),
    }),
  upsertBudget: (input: UpsertBudgetInput) =>
    request<Budget>('/budgets', {
      method: 'PUT',
      body: JSON.stringify(input),
    }),
  budgetSummary: (year?: number, month?: number) => {
    const params = new URLSearchParams()
    if (year != null) params.set('year', String(year))
    if (month != null) params.set('month', String(month))
    const qs = params.toString()
    return request<BudgetSummary>(`/budgets/summary${qs ? `?${qs}` : ''}`)
  },
}

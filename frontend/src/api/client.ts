type Params = Record<string, string | number | boolean | undefined>

/** Where the backend is when it is hosted apart from this page; empty means the same origin. */
const BACKEND = (import.meta.env.VITE_API_URL ?? '').replace(/\/+$/, '')

/** The backend answered, but not with what was asked for. */
export class ApiError extends Error {
  readonly status: number

  constructor(status: number) {
    super(`Request failed (${status})`)
    this.status = status
  }
}

/** GETs JSON from the backend, dropping empty parameters. */
export async function get<T>(path: string, params: Params = {}): Promise<T> {
  const query = new URLSearchParams()
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== '' && value !== false) query.set(key, String(value))
  }
  const response = await fetch(`${BACKEND}/api${path}?${query}`)
  if (!response.ok) throw new ApiError(response.status)
  return response.json() as Promise<T>
}

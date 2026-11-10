type Params = Record<string, string | number | boolean | undefined>

/** GETs JSON from the backend, dropping empty parameters. */
export async function get<T>(path: string, params: Params = {}): Promise<T> {
  const query = new URLSearchParams()
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== '' && value !== false) query.set(key, String(value))
  }
  const response = await fetch(`/api${path}?${query}`)
  if (!response.ok) throw new Error(`Request failed (${response.status})`)
  return response.json() as Promise<T>
}

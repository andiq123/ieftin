export const ROUTES = { search: '/', saved: '/saved', stores: '/stores' } as const

/** Keeps the current search in the URL, so every screen can return to it. */
export const withSearch = (path: string) => path + window.location.search

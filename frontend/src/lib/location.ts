import { useSyncExternalStore } from 'react'
import { flushSync } from 'react-dom'
import { type TransitionKind, prefersReducedMotion, withTransition } from './transition.ts'

/** The browser location as a tiny external store: the app's only routing layer.
 *  Each history entry remembers how deep it is, how it was entered and what was
 *  handed to it, so back and forward can move and scroll the way the app did. */
interface Entry {
  depth: number
  kind: TransitionKind
  data?: unknown
}

const CHANGE_EVENT = 'locationchange'

const readEntry = (): Entry => (window.history.state as Entry | null) ?? { depth: 0, kind: 'page' }
const notify = () => window.dispatchEvent(new Event(CHANGE_EVENT))

let entry = readEntry()
const scrollPositions = new Map<number, number>()
// Restored here instead, once the page being returned to has rendered.
window.history.scrollRestoration = 'manual'

window.addEventListener('popstate', (event) => {
  const next = readEntry()
  const backwards = next.depth < entry.depth
  const kind = backwards ? (entry.kind === 'push' ? 'pop' : entry.kind) : next.kind
  scrollPositions.set(entry.depth, window.scrollY)
  entry = next
  const show = () => {
    // Rendered first: the page must be tall enough to scroll back to where it was.
    flushSync(notify)
    window.scrollTo({ top: scrollPositions.get(next.depth) ?? 0, behavior: 'instant' })
  }
  // A swipe-back gesture has already been animated by the browser.
  if (event.hasUAVisualTransition) show()
  else withTransition(kind, show)
})

export function subscribe(onChange: () => void) {
  window.addEventListener(CHANGE_EVENT, onChange)
  return () => window.removeEventListener(CHANGE_EVENT, onChange)
}

/** Updates the URL in place (no history entry), e.g. when a filter changes. */
export function replaceUrl(url: string) {
  window.history.replaceState(window.history.state, '', url)
  notify()
}

/** Goes to another screen, optionally handing it `data`. Going where we already
 *  are scrolls to the top, like tapping the current tab. */
export function navigate(url: string, kind: TransitionKind = 'page', data?: unknown) {
  if (url === window.location.pathname + window.location.search) {
    return window.scrollTo({ top: 0, behavior: prefersReducedMotion() ? 'instant' : 'smooth' })
  }
  scrollPositions.set(entry.depth, window.scrollY)
  withTransition(kind, () => {
    entry = { depth: entry.depth + 1, kind, data }
    window.history.pushState(entry, '', url)
    notify()
    window.scrollTo({ top: 0, behavior: 'instant' })
  })
}

/** Leaves a pushed screen: back through history when the app led here, otherwise to `fallback`. */
export function goBack(fallback: string) {
  if (entry.depth > 0) window.history.back()
  else navigate(fallback, 'pop')
}

export function usePath(): string {
  return useSyncExternalStore(subscribe, () => window.location.pathname)
}

/** What `navigate` handed to the current screen, if anything. */
export function useEntryData<Data>(): Data | undefined {
  return useSyncExternalStore(subscribe, () => readEntry().data as Data | undefined)
}

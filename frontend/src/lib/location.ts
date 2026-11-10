import { useSyncExternalStore } from 'react'
import { flushSync } from 'react-dom'
import { prefersReducedMotion, withTransition } from './transition.ts'

/** The browser location as a tiny external store: the app's only routing layer.
 *  Each history entry remembers how deep it is, so back and forward can restore
 *  where each screen was scrolled to. */
interface Entry {
  depth: number
}

const CHANGE_EVENT = 'locationchange'

const readEntry = (): Entry => (window.history.state as Entry | null) ?? { depth: 0 }
const notify = () => window.dispatchEvent(new Event(CHANGE_EVENT))

let entry = readEntry()
const scrollPositions = new Map<number, number>()
// Restored here instead, once the page being returned to has rendered.
window.history.scrollRestoration = 'manual'

window.addEventListener('popstate', (event) => {
  const next = readEntry()
  scrollPositions.set(entry.depth, window.scrollY)
  entry = next
  const show = () => {
    // Rendered first: the page must be tall enough to scroll back to where it was.
    flushSync(notify)
    window.scrollTo({ top: scrollPositions.get(next.depth) ?? 0, behavior: 'instant' })
  }
  // A swipe-back gesture has already been animated by the browser.
  if (event.hasUAVisualTransition) show()
  else withTransition(show)
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

/** Goes to another screen. Going where we already are scrolls to the top, like
 *  tapping the current tab. */
export function navigate(url: string) {
  if (url === window.location.pathname + window.location.search) {
    return window.scrollTo({ top: 0, behavior: prefersReducedMotion() ? 'instant' : 'smooth' })
  }
  scrollPositions.set(entry.depth, window.scrollY)
  withTransition(() => {
    entry = { depth: entry.depth + 1 }
    window.history.pushState(entry, '', url)
    notify()
    window.scrollTo({ top: 0, behavior: 'instant' })
  })
}

export function usePath(): string {
  return useSyncExternalStore(subscribe, () => window.location.pathname)
}

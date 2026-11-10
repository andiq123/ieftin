import { flushSync } from 'react-dom'

/** What is changing, which decides how it moves (see styles/motion.css). */
export type TransitionKind = 'page' | 'push' | 'pop' | 'reorder'

export const prefersReducedMotion = () => window.matchMedia('(prefers-reduced-motion: reduce)').matches

/** How long a change may wait for its transition to start, in milliseconds. */
const MAX_WAIT = 400

let latest: ViewTransition | undefined

/** Applies a state change inside a view transition, so elements that moved or
 *  changed animate to their new place. Falls back to a plain update. */
export function withTransition(kind: TransitionKind, update: () => void) {
  if (!document.startViewTransition || prefersReducedMotion()) return update()
  const root = document.documentElement
  root.dataset.transition = kind

  let applied = false
  const apply = () => {
    if (applied) return
    applied = true
    flushSync(update)
  }
  const transition = document.startViewTransition(() => {
    clearTimeout(overdue)
    apply()
  })
  latest = transition
  // A transition starts on the next painted frame. A page that is not being
  // painted never gets one, and the change must not wait for it forever.
  const overdue = setTimeout(() => {
    apply()
    transition.skipTransition()
  }, MAX_WAIT)
  // A newer transition may have taken over; only the last one clears the marker.
  const clear = () => {
    if (latest === transition) delete root.dataset.transition
  }
  transition.finished.then(clear, clear)
  // Being skipped (a newer transition, a hidden tab) is not an error.
  transition.ready.catch(() => undefined)
}

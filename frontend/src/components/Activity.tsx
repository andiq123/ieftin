import type { Store } from '../api/types.ts'
import { Icon } from './Icon.tsx'

/** What is running right now, in a few words. */
export function Activity({ stores }: { stores: Store[] }) {
  const count = (state: Store['activity']['state']) => stores.filter((store) => store.activity.state === state).length
  const [checking, crawling] = [count('checking'), count('crawling')]
  const working = [checking > 0 && `${checking} checking`, crawling > 0 && `${crawling} crawling`].filter(Boolean)
  return (
    <p className="activity navbar__end" aria-live="polite">
      {working.length > 0 ? <span className="pulse" /> : <Icon name="check" className="tone tone--ok" />}
      {working.join(' · ') || 'Up to date'}
    </p>
  )
}

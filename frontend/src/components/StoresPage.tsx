import type { ReactNode } from 'react'
import type { Store } from '../api/types.ts'
import { relativeTime } from '../lib/format.ts'
import { STATUSES, isSearched, statusOf } from '../lib/stores.ts'
import { Icon, type IconName } from './Icon.tsx'

/** Every store: those asked on each search, and those that cannot be used right now. */
export function StoresPage({ stores }: { stores: Store[] }) {
  const searched = stores.filter(isSearched)
  const failing = stores.filter((store) => !isSearched(store))

  return (
    <section className="stagger">
      <h1 className="large-title">Stores</h1>

      <div className="status-bar" aria-hidden="true">
        <span className="status-bar__part tone--busy" style={{ flexGrow: searched.length }} />
        <span className="status-bar__part tone--pending" style={{ flexGrow: failing.length }} />
      </div>

      <Group icon="bolt" tone="busy" title="Searched" count={searched.length} note="Asked on every search">
        <ul className="tiles">
          {searched.map((store) => (
            <li key={store.id} className="tile">
              <StoreLink store={store} />
              <span className="muted">
                {store.checking ? 'testing…' : store.checked_at ? `tested ${relativeTime(store.checked_at)}` : 'queued'}
              </span>
              <Icon
                name={store.checking ? 'spinner' : STATUSES[statusOf(store)].icon}
                className="tone"
                label={store.checking ? 'Being tested' : STATUSES[statusOf(store)].label}
              />
            </li>
          ))}
        </ul>
      </Group>

      <Group icon="empty" tone="pending" title="Not working" count={failing.length} note="Left out, retried regularly">
        <ul className="list">
          {failing.map((store) => {
            const status = statusOf(store)
            return (
              <li key={store.id} className={`inactive tone--${status}`} title={store.detail ?? undefined}>
                <span className="glyph">
                  <Icon name={store.checking ? 'spinner' : STATUSES[status].icon} />
                </span>
                <StoreLink store={store} />
                <span className="muted">{STATUSES[status].label}</span>
                <span className="fact">
                  <Icon name="clock" label="Next test" />
                  {store.checking ? 'testing…' : store.next_check_at ? relativeTime(store.next_check_at) : 'soon'}
                </span>
              </li>
            )
          })}
        </ul>
      </Group>
    </section>
  )
}

interface GroupProps {
  icon: IconName
  tone: string
  title: string
  count: number
  note: string
  children: ReactNode
}

/** A heading that says what the group is, then its stores. */
function Group({ icon, tone, title, count, note, children }: GroupProps) {
  if (count === 0) return null
  return (
    <section className={`group tone--${tone}`}>
      <header className="group__head">
        <h2>
          <span className="glyph">
            <Icon name={icon} />
          </span>
          {title}
          <span className="group__count">{count}</span>
        </h2>
        <p className="group__note">{note}</p>
      </header>
      {children}
    </section>
  )
}

function StoreLink({ store }: { store: Store }) {
  return (
    <a className="store-link" href={store.url} target="_blank" rel="noreferrer">
      {store.name}
      <Icon name="external" />
    </a>
  )
}

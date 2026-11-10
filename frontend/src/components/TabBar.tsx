import { type CSSProperties, useState } from 'react'
import { ROUTES, withSearch } from '../lib/routes.ts'
import { Icon, type IconName } from './Icon.tsx'
import { Link } from './Link.tsx'

const TABS = [
  { id: 'search', label: 'Search', icon: 'search', path: ROUTES.search },
  { id: 'saved', label: 'Saved', icon: 'bookmark', path: ROUTES.saved },
  { id: 'stores', label: 'Stores', icon: 'stores', path: ROUTES.stores },
] as const satisfies { id: string; label: string; icon: IconName; path: string }[]

export type Tab = (typeof TABS)[number]['id']

interface Props {
  current: Tab
  /** Background work is running for some store. */
  busy: boolean
}

/** The sections of the app. Every link carries the current search along, so
 *  switching tabs and coming back loses nothing. */
export function TabBar({ current, busy }: Props) {
  // The tapped tab lights up at once, ahead of the page transition; the route
  // takes over again whenever it changes (also by back and forward).
  const [chosen, setChosen] = useState(current)
  const [route, setRoute] = useState(current)
  if (route !== current) {
    setRoute(current)
    setChosen(current)
  }
  const position = {
    '--count': TABS.length,
    '--index': TABS.findIndex((tab) => tab.id === chosen),
  } as CSSProperties

  return (
    <nav className="tabbar" aria-label="Sections" style={position}>
      <span className="tabbar__thumb" aria-hidden="true" />
      {TABS.map(({ id, label, icon, path }) => (
        <Link
          key={id}
          className="tabbar__tab"
          to={withSearch(path)}
          current={id === chosen}
          onFollow={() => setChosen(id)}
        >
          <Icon name={icon} />
          {id === 'stores' && busy && <span className="pulse" title="Updating stores" />}
          {label}
        </Link>
      ))}
    </nav>
  )
}

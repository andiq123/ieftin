import type { Store } from '../api/types.ts'
import { formatCount } from '../lib/format.ts'
import { ROUTES } from '../lib/routes.ts'
import { isActive, isLive, totalProducts } from '../lib/stores.ts'
import { Chip } from './Chip.tsx'
import { Icon, type IconName } from './Icon.tsx'
import { Link } from './Link.tsx'

const EXAMPLES = ['iPhone 15 Pro', 'Samsung Galaxy S24', 'MacBook Air', 'PlayStation 5']

interface Props {
  stores: Store[]
  onExample: (query: string) => void
}

/** Start screen: example searches and how much is being compared. */
export function Home({ stores, onExample }: Props) {
  const active = stores.filter(isActive)
  const live = active.filter(isLive).length

  return (
    <section className="home stagger">
      <header>
        <h1 className="large-title">Who sells it cheaper?</h1>
        <p className="subtitle">Prices across Moldova, lowest first.</p>
      </header>

      <ul className="chips" aria-label="Example searches">
        {EXAMPLES.map((example) => (
          <li key={example}>
            <Chip onClick={() => onExample(example)}>
              <Icon name="search" />
              {example}
            </Chip>
          </li>
        ))}
      </ul>

      {stores.length > 0 && (
        <ul className="list" aria-label="Coverage">
          <Stat icon="bolt" tone="busy" label="Live stores" value={live} />
          <Stat icon="database" tone="ok" label="Daily stores" value={active.length - live} />
          <Stat icon="tag" tone="no_data" label="Products" value={totalProducts(active)} />
          <li>
            <Link className="row tone--pending" to={ROUTES.stores}>
              <span className="glyph">
                <Icon name="stores" />
              </span>
              <span className="row__label">All stores</span>
              <Icon name="chevron" />
            </Link>
          </li>
        </ul>
      )}
    </section>
  )
}

interface StatProps {
  icon: IconName
  tone: string
  label: string
  value: number
}

function Stat({ icon, tone, label, value }: StatProps) {
  return (
    <li className={`row tone--${tone}`}>
      <span className="glyph">
        <Icon name={icon} />
      </span>
      <span className="row__label">{label}</span>
      <span className="row__value">{formatCount(value)}</span>
    </li>
  )
}

import type { Store } from '../api/types.ts'
import { ROUTES } from '../lib/routes.ts'
import { isSearched } from '../lib/stores.ts'
import { Chip } from './Chip.tsx'
import { Icon } from './Icon.tsx'
import { Link } from './Link.tsx'

const EXAMPLES = ['iPhone 15 Pro', 'Samsung Galaxy S24', 'MacBook Air', 'PlayStation 5']

interface Props {
  stores: Store[]
  onExample: (query: string) => void
}

/** Start screen: example searches and how many stores are compared. */
export function Home({ stores, onExample }: Props) {
  const searched = stores.filter(isSearched).length

  return (
    <section className="home stagger">
      <header>
        <h1 className="large-title">Who sells it cheaper?</h1>
        <p className="subtitle">Live prices from Moldova's stores, lowest first.</p>
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
          <li>
            <Link className="row tone--busy" to={ROUTES.stores}>
              <span className="glyph">
                <Icon name="bolt" />
              </span>
              <span className="row__label">Stores asked on every search</span>
              <span className="row__value">{searched}</span>
              <Icon name="chevron" />
            </Link>
          </li>
        </ul>
      )}
    </section>
  )
}

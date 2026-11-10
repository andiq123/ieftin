import { useStores } from './api/queries.ts'
import { ConnectionToast } from './components/ConnectionToast.tsx'
import { Home } from './components/Home.tsx'
import { Link } from './components/Link.tsx'
import { Results } from './components/Results.tsx'
import { SavedPage } from './components/SavedPage.tsx'
import { SearchBar } from './components/SearchBar.tsx'
import { StoresPage } from './components/StoresPage.tsx'
import { TabBar } from './components/TabBar.tsx'
import { useFilters } from './lib/filters.ts'
import { usePath } from './lib/location.ts'
import { ROUTES } from './lib/routes.ts'
import { isBusy, storeNamer } from './lib/stores.ts'
import { withTransition } from './lib/transition.ts'

export function App() {
  const path = usePath()
  const onStoresPage = path === ROUTES.stores
  const onSavedPage = path === ROUTES.saved
  const [filters, updateFilters] = useFilters()
  const stores = useStores().data ?? []
  const searching = filters.q.trim() !== ''
  const storeName = storeNamer(stores)

  const search = (q: string) => {
    const update = () => {
      updateFilters({ q, store: undefined, category: undefined })
      window.scrollTo({ top: 0, behavior: 'instant' })
    }
    // Between home and results the screen changes; a new search over old results
    // has its own loading state instead.
    if ((q !== '') === searching) update()
    else withTransition(update)
  }

  return (
    <>
      <header className="navbar">
        <Link className="wordmark" to={ROUTES.search}>
          ieftin
        </Link>
        {!onStoresPage && !onSavedPage && <SearchBar query={filters.q} onSearch={search} />}
      </header>

      <main className="page">
        {onSavedPage ? (
          <SavedPage storeName={storeName} />
        ) : onStoresPage ? (
          <StoresPage stores={stores} />
        ) : searching ? (
          <Results filters={filters} updateFilters={updateFilters} stores={stores} />
        ) : (
          <Home stores={stores} onExample={search} />
        )}
      </main>

      <ConnectionToast />
      <TabBar current={onStoresPage ? 'stores' : onSavedPage ? 'saved' : 'search'} busy={stores.some(isBusy)} />
    </>
  )
}

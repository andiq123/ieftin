import { useStores } from '../api/queries.ts'
import { useOnline } from '../lib/connection.ts'
import { Icon } from './Icon.tsx'

/** Says so when the backend cannot be reached, and goes away by itself once it can.
 *  The stores list is polled on every screen, which makes it the app's heartbeat. */
export function ConnectionToast() {
  const online = useOnline()
  const stores = useStores()
  // The last answer decides, not the status: a retry in flight is still "unreachable".
  const reachable = stores.errorUpdatedAt <= stores.dataUpdatedAt
  if (online && reachable) return null

  return (
    <div className="toast" role="alert">
      <Icon name="offline" />
      <div className="toast__text">
        <strong>{online ? 'Can’t reach Ieftin' : 'You’re offline'}</strong>
        <span>{online ? 'Prices may be out of date. Retrying…' : 'Check your internet connection.'}</span>
      </div>
      {online && (
        <button className="toast__action" type="button" disabled={stores.isFetching} onClick={() => stores.refetch()}>
          {stores.isFetching ? <Icon name="spinner" /> : 'Retry'}
        </button>
      )}
    </div>
  )
}

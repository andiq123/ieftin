import { useFavorites } from '../lib/favorites.ts'
import { formatDay } from '../lib/format.ts'
import { Notice } from './Notice.tsx'
import { OfferRow } from './OfferRow.tsx'

/** Offers saved on this device, as they were when saved. Opening one shows the
 *  store's page, where the price is always the current one. */
export function SavedPage({ storeName }: { storeName: (id: string) => string }) {
  const favorites = useFavorites()
  return (
    <section className="saved stagger">
      <h1 className="large-title">Saved</h1>
      {favorites.length === 0 ? (
        <Notice icon="bookmark" title="Nothing saved yet">
          Save an offer from a search to keep it here.
        </Notice>
      ) : (
        <ul className="list offers">
          {favorites.map(({ offer, savedAt }) => (
            <OfferRow
              key={offer.url}
              offer={offer}
              storeName={storeName(offer.store_id)}
              saved
              note={`price on ${formatDay(savedAt)}`}
            />
          ))}
        </ul>
      )}
    </section>
  )
}

import { useHistory } from '../api/queries.ts'
import { type Favorite, useFavorites } from '../lib/favorites.ts'
import { formatDay } from '../lib/format.ts'
import { Notice } from './Notice.tsx'
import { Change } from './OfferParts.tsx'
import { OfferRow } from './OfferRow.tsx'

/** Offers saved on this device, each with how its price moved since it was saved. */
export function SavedPage({ storeName }: { storeName: (id: string) => string }) {
  const favorites = useFavorites()
  return (
    <section className="saved stagger">
      <h1 className="large-title">Saved</h1>
      {favorites.length === 0 ? (
        <Notice icon="bookmark" title="Nothing saved yet">
          Save an offer to follow its price here.
        </Notice>
      ) : (
        <ul className="list offers">
          {favorites.map((favorite) => (
            <SavedOffer key={favorite.offer.url} favorite={favorite} storeName={storeName(favorite.offer.store_id)} />
          ))}
        </ul>
      )}
    </section>
  )
}

function SavedOffer({ favorite, storeName }: { favorite: Favorite; storeName: string }) {
  const history = useHistory(favorite.offer).data
  const savedPrice = favorite.offer.price_bani
  // The index has the offer as it is now; otherwise the last recorded price stands in.
  const current = history?.offer ?? {
    ...favorite.offer,
    price_bani: history?.points.at(-1)?.price_bani ?? savedPrice,
  }
  return (
    <OfferRow
      offer={current}
      storeName={storeName}
      saved
      note={
        <>
          {current.price_bani !== savedPrice && <Change from={savedPrice} to={current.price_bani} />}
          since {formatDay(favorite.savedAt)}
        </>
      }
    />
  )
}

import type { ReactNode } from 'react'
import type { Offer } from '../api/types.ts'
import { toggleFavorite } from '../lib/favorites.ts'
import { formatLei, percentChange } from '../lib/format.ts'
import type { Badge } from '../lib/title.ts'
import { Icon } from './Icon.tsx'

/** The product photo on a white tile, or a placeholder symbol when the store has none. */
export function OfferImage({ offer, className }: { offer: Offer; className: string }) {
  return (
    <div className={className}>
      {offer.image_url ? (
        <img src={offer.image_url} alt="" loading="lazy" referrerPolicy="no-referrer" />
      ) : (
        <Icon name="tag" />
      )}
    </div>
  )
}

interface MetaProps {
  offer: Offer
  storeName: string
  /** Condition markers found in the title. */
  badges: Badge[]
}

/** Where an offer is sold, and what is worth knowing before opening it. */
export function OfferMeta({ offer, storeName, badges }: MetaProps) {
  return (
    <p className="offer__meta">
      <span className="offer__store">{storeName}</span>
      {badges.map(({ icon, label }) => (
        <span key={label} className="badge">
          <Icon name={icon} />
          {label}
        </span>
      ))}
      {offer.category && <span>{offer.category}</span>}
      {offer.in_stock === false && (
        <span className="offer__out">
          <Icon name="empty" />
          Out of stock
        </span>
      )}
    </p>
  )
}

/** The price, the crossed-out old price and discount when there is one, then any note. */
export function OfferPrice({ offer, children }: { offer: Offer; children?: ReactNode }) {
  const discounted = offer.old_price_bani !== null && offer.old_price_bani > offer.price_bani
  return (
    <div className="offer__price">
      <strong>
        {formatLei(offer.price_bani)} <small>lei</small>
      </strong>
      {discounted && (
        <>
          <s>{formatLei(offer.old_price_bani!)} lei</s>
          <span className="badge">{percentChange(offer.old_price_bani!, offer.price_bani)}%</span>
        </>
      )}
      {children}
    </div>
  )
}

interface SaveProps {
  offer: Offer
  saved: boolean
}

/** Saves the offer on this device, or removes it. */
export function SaveButton({ offer, saved }: SaveProps) {
  const label = saved ? 'Remove from saved' : 'Save'
  return (
    <button
      className="icon-button"
      type="button"
      aria-pressed={saved}
      aria-label={label}
      title={label}
      onClick={() => toggleFavorite(offer)}
    >
      <Icon name="bookmark" className={saved ? 'icon--filled' : undefined} />
    </button>
  )
}

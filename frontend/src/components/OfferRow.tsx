import type { ReactNode } from 'react'
import type { Offer } from '../api/types.ts'
import { offerPath, withSearch } from '../lib/routes.ts'
import { parseTitle } from '../lib/title.ts'
import { Icon } from './Icon.tsx'
import { Link } from './Link.tsx'
import { OfferImage, OfferMeta, OfferPrice, SaveButton } from './OfferParts.tsx'

interface Props {
  offer: Offer
  storeName: string
  saved: boolean
  /** Position in a ranked list. */
  rank?: number
  /** Carries the price tag: the cheapest offer of the list. */
  cheapest?: boolean
  /** A short remark under the price. */
  note?: ReactNode
}

/** One offer in a list. The row opens the store; its buttons open the price
 *  history and save the offer. */
export function OfferRow({ offer, storeName, saved, rank, cheapest, note }: Props) {
  const { title, badges } = parseTitle(offer.title)

  return (
    <li className={cheapest ? 'offer offer--cheapest' : 'offer'}>
      {rank !== undefined && (
        <span className="offer__rank" aria-hidden="true">
          {rank}
        </span>
      )}
      <OfferImage offer={offer} className="offer__image" />
      <div className="offer__body">
        <a className="offer__title" href={offer.url} target="_blank" rel="noreferrer">
          {title}
        </a>
        <OfferMeta offer={offer} storeName={storeName} badges={badges} />
      </div>
      <OfferPrice offer={offer}>{note && <span className="offer__note">{note}</span>}</OfferPrice>
      <div className="offer__actions">
        <Link className="icon-button" to={withSearch(offerPath(offer))} kind="push" data={offer} label="Price history">
          <Icon name="chart" />
        </Link>
        <SaveButton className="icon-button" offer={offer} saved={saved} />
      </div>
    </li>
  )
}

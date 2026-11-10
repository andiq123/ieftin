import type { ReactNode } from 'react'
import { useHistory } from '../api/queries.ts'
import type { Offer } from '../api/types.ts'
import { savedLookup, useFavorites } from '../lib/favorites.ts'
import { formatDay, formatLei, relativeTime } from '../lib/format.ts'
import { type Sample, type Summary, percentChange, summarize, toSamples } from '../lib/history.ts'
import { goBack, useEntryData } from '../lib/location.ts'
import { type OfferKey, ROUTES, withSearch } from '../lib/routes.ts'
import { parseTitle } from '../lib/title.ts'
import { Icon, type IconName } from './Icon.tsx'
import { Notice } from './Notice.tsx'
import { Change, OfferImage, OfferMeta, OfferPrice, SaveButton } from './OfferParts.tsx'
import { PriceHistory } from './PriceHistory.tsx'

const MAX_CHANGES = 20

interface Props {
  offerKey: OfferKey
  storeName: string
}

/** One offer and how its price has moved. Opened from a list, it shows the offer
 *  it was handed at once; opened from a link, it waits for the index to supply it. */
export function OfferPage({ offerKey, storeName }: Props) {
  const handed = useEntryData<Offer>()
  const history = useHistory(offerKey)
  const isSaved = savedLookup(useFavorites())
  // The index knows the price now; what was handed over may be an older saved copy.
  const offer = history.data?.offer ?? handed

  if (!offer) {
    if (history.isPending) return <Notice icon="spinner" title="Loading…" />
    return (
      <Notice
        icon="empty"
        title="Offer not found"
        alert={history.isError}
        action={{ label: 'Back to search', onClick: () => goBack(withSearch(ROUTES.search)) }}
      >
        It may have sold out. Search again to see current offers.
      </Notice>
    )
  }

  const samples = toSamples(history.data?.points ?? [])
  const { title, badges } = parseTitle(offer.title)

  return (
    <article className="offer-page stagger">
      <header className="hero">
        <OfferImage offer={offer} className="hero__image" />
        <div>
          <OfferMeta offer={offer} storeName={storeName} badges={badges} />
          <h1>{title}</h1>
          <OfferPrice offer={offer} />
        </div>
      </header>

      <div className="offer-page__actions">
        <a className="button button--primary" href={offer.url} target="_blank" rel="noreferrer">
          Open in {storeName}
          <Icon name="external" />
        </a>
        <SaveButton className="button" offer={offer} saved={isSaved(offer)} labelled />
      </div>

      {samples.length > 0 ? (
        <History samples={samples} />
      ) : history.isPending ? (
        <div className="card history" aria-busy="true">
          <Icon name="spinner" />
        </div>
      ) : (
        <Insight tone="pending" icon="clock">
          No price history yet
        </Insight>
      )}
    </article>
  )
}

function History({ samples }: { samples: Sample[] }) {
  const summary = summarize(samples)
  // Latest first, each with the price it replaced.
  const changes = samples.map((sample, index) => ({ sample, before: samples[index - 1] })).reverse()

  return (
    <>
      <Verdict summary={summary} />
      <PriceHistory samples={samples} />

      {changes.length > 1 && (
        <>
          <ul className="list" aria-label="Price range">
            <Extreme tone="ok" icon="arrow-down" label="Lowest" sample={summary.lowest} />
            <Extreme tone="no_data" icon="arrow-up" label="Highest" sample={summary.highest} />
            <li className="row tone--pending">
              <span className="glyph">
                <Icon name="clock" />
              </span>
              <span className="row__label">Tracked since</span>
              <span className="row__value">{formatDay(summary.since)}</span>
            </li>
          </ul>
          <section>
            <h2 className="section-title">Changes</h2>
            <ol className="list">
              {changes.slice(0, MAX_CHANGES).map(({ sample, before }) => (
                <li key={sample.time} className="row">
                  <span className="row__label">{formatDay(sample.time)}</span>
                  {before && <Change from={before.price} to={sample.price} />}
                  <span className="row__value">{formatLei(sample.price)} lei</span>
                </li>
              ))}
            </ol>
          </section>
        </>
      )}
    </>
  )
}

/** What the history says about buying now, in one line. */
function Verdict({ summary }: { summary: Summary }) {
  const { current, previous, lowest } = summary
  if (!previous) {
    return (
      <Insight tone="pending" icon="clock">
        Tracking since {formatDay(summary.since)}
      </Insight>
    )
  }
  if (current.price <= lowest.price) {
    return (
      <Insight tone="ok" icon="tag">
        Lowest price so far
      </Insight>
    )
  }
  const percent = percentChange(previous.price, current.price)
  return percent < 0 ? (
    <Insight tone="ok" icon="arrow-down">
      Down {-percent}% {relativeTime(current.time)}
    </Insight>
  ) : (
    <Insight tone="no_data" icon="arrow-up">
      Up {percent}% {relativeTime(current.time)}, was {formatLei(lowest.price)} lei
    </Insight>
  )
}

interface InsightProps {
  tone: string
  icon: IconName
  children: ReactNode
}

function Insight({ tone, icon, children }: InsightProps) {
  return (
    <p className={`card insight tone--${tone}`}>
      <span className="glyph">
        <Icon name={icon} />
      </span>
      {children}
    </p>
  )
}

interface ExtremeProps {
  tone: string
  icon: IconName
  label: string
  sample: Sample
}

function Extreme({ tone, icon, label, sample }: ExtremeProps) {
  return (
    <li className={`row tone--${tone}`}>
      <span className="glyph">
        <Icon name={icon} />
      </span>
      <span className="row__label">{label}</span>
      <span className="row__value">
        {formatLei(sample.price)} lei · {formatDay(sample.time)}
      </span>
    </li>
  )
}

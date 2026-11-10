import type { PointerEvent } from 'react'
import { formatDay, formatLei } from '../lib/format.ts'
import { type Sample, priceAt } from '../lib/history.ts'

interface Props {
  /** Price changes in time order; the first and last mark the edges of the chart. */
  samples: Sample[]
  /** The moment under the pointer, drawn as a rule and a dot. */
  scrubbed: Sample | undefined
  onScrub: (sample: Sample | undefined) => void
}

/** A step line of one price over time: a price holds until the next change.
 *  The drawing is stretched to its box, so positions are percentages throughout. */
export function PriceChart({ samples, scrubbed, onScrub }: Props) {
  const [start, end] = [samples[0].time, samples.at(-1)!.time]
  const prices = samples.map((sample) => sample.price)
  const [low, high] = [Math.min(...prices), Math.max(...prices)]
  // Room above and below, so the line never rides the edge; a flat price sits mid-height.
  const margin = (high - low || high * 0.1) * 0.25
  const x = (time: number) => ((time - start) / (end - start || 1)) * 100
  const y = (price: number) => ((high + margin - price) / (high - low + 2 * margin)) * 100

  const line = samples
    .map(({ time, price }, index) => (index === 0 ? `M${x(time)} ${y(price)}` : `H${x(time)}V${y(price)}`))
    .join('')
  const levels = low === high ? [high] : [high, low]

  const scrub = (event: PointerEvent) => {
    const box = event.currentTarget.getBoundingClientRect()
    const share = Math.min(1, Math.max(0, (event.clientX - box.left) / box.width))
    const time = start + share * (end - start)
    onScrub({ time, price: priceAt(samples, time).price })
  }
  const release = () => onScrub(undefined)

  return (
    <div
      className="chart"
      role="img"
      aria-label={`Price since ${formatDay(start)}: ${formatLei(low)} to ${formatLei(high)} lei`}
      onPointerDown={scrub}
      onPointerMove={scrub}
      onPointerLeave={release}
      onPointerCancel={release}
    >
      <svg viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
        <defs>
          <linearGradient id="chart-fill" x1="0" y1="0" x2="0" y2="1">
            <stop className="chart__fill-start" offset="0" />
            <stop className="chart__fill-end" offset="1" />
          </linearGradient>
        </defs>
        {levels.map((price) => (
          <line key={price} className="chart__grid" x1="0" x2="100" y1={y(price)} y2={y(price)} />
        ))}
        <path className="chart__area" d={`${line}V100H0Z`} />
        <path className="chart__line" d={line} />
      </svg>
      {levels.map((price) => (
        <span
          key={price}
          className={price === high ? 'chart__label' : 'chart__label chart__label--below'}
          style={{ top: `${y(price)}%` }}
        >
          {formatLei(price)}
        </span>
      ))}
      {scrubbed && (
        <>
          <span className="chart__rule" style={{ left: `${x(scrubbed.time)}%` }} />
          <span className="chart__dot" style={{ left: `${x(scrubbed.time)}%`, top: `${y(scrubbed.price)}%` }} />
        </>
      )}
    </div>
  )
}

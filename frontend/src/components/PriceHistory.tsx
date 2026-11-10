import { useState } from 'react'
import { formatDay, formatLei } from '../lib/format.ts'
import { type Range, type Sample, availableRanges, within } from '../lib/history.ts'
import { PriceChart } from './PriceChart.tsx'
import { Segmented } from './Segmented.tsx'

/** The chart card: a readout that follows the pointer, a range switch once the
 *  history is long enough to need one, and the chart itself. */
export function PriceHistory({ samples }: { samples: Sample[] }) {
  const [now] = useState(Date.now)
  const [range, setRange] = useState<Range>('all')
  const [scrubbed, setScrubbed] = useState<Sample>()
  const ranges = availableRanges(samples, now)
  const visible = within(samples, range, now)
  const shown = scrubbed ?? visible.at(-1)!

  return (
    <section className="card history" aria-label="Price history">
      <header className="history__head">
        <p className="history__readout">
          <strong>{formatLei(shown.price)} lei</strong>
          <span>{scrubbed ? formatDay(scrubbed.time) : 'Now'}</span>
        </p>
        {ranges.length > 1 && <Segmented label="Range" options={ranges} value={range} onChange={setRange} />}
      </header>
      {/* Keyed by range, so the line draws itself again for each one. */}
      <PriceChart key={range} samples={visible} scrubbed={scrubbed} onScrub={setScrubbed} />
      <p className="chart__axis">
        <span>{formatDay(visible[0].time)}</span>
        <span>Today</span>
      </p>
    </section>
  )
}

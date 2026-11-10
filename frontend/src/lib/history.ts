import type { PricePoint } from '../api/types.ts'

/** A price and the moment it applies to, in bani and epoch milliseconds. */
export interface Sample {
  time: number
  price: number
}

const DAY = 86_400_000

export const RANGES = [
  { value: 'week', label: '1W', days: 7 },
  { value: 'month', label: '1M', days: 30 },
  { value: 'quarter', label: '3M', days: 90 },
  { value: 'all', label: 'All', days: Infinity },
] as const

export type Range = (typeof RANGES)[number]['value']

export const toSamples = (points: PricePoint[]): Sample[] =>
  points.map((point) => ({ time: Date.parse(point.observed_at), price: point.price_bani }))

/** The ranges worth offering: those the history is long enough to fill, and "All". */
export function availableRanges(samples: Sample[], now: number) {
  const tracked = now - samples[0].time
  return RANGES.filter((range) => range.days === Infinity || range.days * DAY < tracked)
}

/** The price changes inside a range, opening with the price already in force when
 *  it starts and closing with today's, so the line spans the whole range. */
export function within(samples: Sample[], range: Range, now: number): Sample[] {
  const { days } = RANGES.find((candidate) => candidate.value === range)!
  const start = Math.max(samples[0].time, now - days * DAY)
  const before = samples.findLast((sample) => sample.time <= start)!
  const inside = samples.filter((sample) => sample.time > start)
  return [{ time: start, price: before.price }, ...inside, { time: now, price: samples.at(-1)!.price }]
}

/** The sample in force at `time`: prices hold until the next change. */
export const priceAt = (samples: Sample[], time: number): Sample =>
  samples.findLast((sample) => sample.time <= time) ?? samples[0]

export interface Summary {
  current: Sample
  /** The price before the latest change, when there has been one. */
  previous: Sample | undefined
  lowest: Sample
  highest: Sample
  since: number
}

export function summarize(samples: Sample[]): Summary {
  const pick = (better: (a: number, b: number) => boolean) =>
    samples.reduce((best, sample) => (better(sample.price, best.price) ? sample : best))
  return {
    current: samples.at(-1)!,
    previous: samples.at(-2),
    lowest: pick((a, b) => a < b),
    highest: pick((a, b) => a > b),
    since: samples[0].time,
  }
}

/** Signed change from one price to another, in whole percent. */
export const percentChange = (from: number, to: number) => Math.round(((to - from) / from) * 100)

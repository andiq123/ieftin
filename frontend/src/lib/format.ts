const number = new Intl.NumberFormat('en-US', { maximumFractionDigits: 2 })
const relative = new Intl.RelativeTimeFormat('en', { numeric: 'auto', style: 'narrow' })
const day = new Intl.DateTimeFormat('en-GB', { day: 'numeric', month: 'short' })
const dayWithYear = new Intl.DateTimeFormat('en-GB', { day: 'numeric', month: 'short', year: 'numeric' })

/** 17349 -> "17 349": a space groups thousands, as prices are written in Moldova. */
export function formatCount(count: number): string {
  return number.format(count).replace(/,/g, ' ')
}

/** 1734900 -> "17 349" (the currency label is rendered separately). */
export function formatLei(bani: number): string {
  return formatCount(bani / 100)
}

/** A moment relative to now: "5m ago", "in 2h", "yesterday", … */
export function relativeTime(time: string | number): string {
  const minutes = Math.round((new Date(time).getTime() - Date.now()) / 60_000)
  if (Math.abs(minutes) < 60) return relative.format(minutes, 'minute')
  const hours = Math.round(minutes / 60)
  return Math.abs(hours) < 24 ? relative.format(hours, 'hour') : relative.format(Math.round(hours / 24), 'day')
}

/** A calendar day: "3 Nov", with the year when it is not this one. */
export function formatDay(time: string | number): string {
  const date = new Date(time)
  return (date.getFullYear() === new Date().getFullYear() ? day : dayWithYear).format(date)
}

/** A rough remaining time: "<1 min", "~12 min", "~3 h". */
export function formatEta(seconds: number): string {
  if (seconds < 60) return '<1 min'
  const minutes = Math.round(seconds / 60)
  return minutes < 90 ? `~${minutes} min` : `~${Math.round(minutes / 60)} h`
}

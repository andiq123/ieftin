import type { IconName } from '../components/Icon.tsx'

/** Something a store wrote into a product title that reads better as a badge. */
export interface Badge {
  icon: IconName
  label: string
}

/** Condition markers stores put in titles, e.g. "{Grade A+} Refurb." or "{Ref.}". */
const MARKERS: { pattern: RegExp; badge: (match: RegExpExecArray) => Badge }[] = [
  {
    pattern: /\{?\s*\bgrade\s*([a-d][+-]?)(?![a-z])\s*\}?/i,
    badge: (match) => ({ icon: 'star', label: `Grade ${match[1].toUpperCase()}` }),
  },
  {
    pattern: /\{\s*ref\.?\s*\}|\{?\s*\b(?:refurb(?:ished|\.)?|recondi[tț]ionat[aă]?)(?![a-z])\s*\}?/i,
    badge: () => ({ icon: 'crawl', label: 'Refurbished' }),
  },
  {
    pattern: /\{?\s*\b(?:resigilat[aă]?|open[\s-]?box)\b\s*\}?/i,
    badge: () => ({ icon: 'box', label: 'Open box' }),
  },
  {
    pattern: /\{?\s*\b(?:second[\s-]?hand|used)(?![\w-])\s*\}?/i,
    badge: () => ({ icon: 'clock', label: 'Used' }),
  },
]

/** Splits a store's title into the product name and the badges hidden in it. */
export function parseTitle(raw: string): { title: string; badges: Badge[] } {
  const badges: Badge[] = []
  let title = raw
  for (const { pattern, badge } of MARKERS) {
    const match = pattern.exec(title)
    if (!match) continue
    badges.push(badge(match))
    title = title.replace(pattern, ' ')
  }
  title = title
    .replace(/\s{2,}/g, ' ')
    .replace(/[\s,;-]+$/, '')
    .trim()
  // A title that was nothing but markers stays as the store wrote it.
  return title ? { title, badges } : { title: raw, badges: [] }
}

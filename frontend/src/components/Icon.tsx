import type { ReactNode } from 'react'

/** The app's icon set: 24px stroke icons, sized by font-size, coloured by currentColor. */
const PATHS = {
  check: <path d="M20 6 9 17l-5-5" />,
  spinner: <path d="M21 12a9 9 0 1 1-6.2-8.56" />,
  lock: (
    <>
      <rect x="4" y="11" width="16" height="10" rx="2" />
      <path d="M8 11V7a4 4 0 0 1 8 0v4" />
    </>
  ),
  empty: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="m5.6 5.6 12.8 12.8" />
    </>
  ),
  offline: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="m15 9-6 6M9 9l6 6" />
    </>
  ),
  clock: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="M12 7v5l3 2" />
    </>
  ),
  external: <path d="M7 17 17 7M8 7h9v9" />,
  search: (
    <>
      <circle cx="11" cy="11" r="7" />
      <path d="m20 20-3.6-3.6" />
    </>
  ),
  stores: (
    <>
      <path d="M4 9.5 5.5 4h13L20 9.5" />
      <path d="M4 9.5a2.7 2.7 0 0 0 5.3 0 2.7 2.7 0 0 0 5.4 0 2.7 2.7 0 0 0 5.3 0" />
      <path d="M5.5 12.5V20h13v-7.5" />
    </>
  ),
  crawl: (
    <>
      <path d="M20 12a8 8 0 0 0-14.3-4.9L4 9" />
      <path d="M4 4v5h5M4 12a8 8 0 0 0 14.3 4.9L20 15" />
      <path d="M20 20v-5h-5" />
    </>
  ),
  database: (
    <>
      <ellipse cx="12" cy="6" rx="7" ry="3" />
      <path d="M5 6v6c0 1.7 3.1 3 7 3s7-1.3 7-3V6M5 12v6c0 1.7 3.1 3 7 3s7-1.3 7-3v-6" />
    </>
  ),
  bolt: <path d="M13 3 5 13.5h6L10 21l8-10.5h-6L13 3Z" />,
  tag: (
    <>
      <path d="M3.5 12.6V4.5a1 1 0 0 1 1-1h8.1a1 1 0 0 1 .7.3l7 7a1 1 0 0 1 0 1.4l-8.1 8.1a1 1 0 0 1-1.4 0l-7-7a1 1 0 0 1-.3-.7Z" />
      <path d="M8 8h.01" />
    </>
  ),
  alert: (
    <>
      <path d="M12 4 2.8 19.5h18.4L12 4Z" />
      <path d="M12 10v4M12 17.2v.1" />
    </>
  ),
  eye: (
    <>
      <path d="M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12Z" />
      <circle cx="12" cy="12" r="3" />
    </>
  ),
  'eye-off': (
    <>
      <path d="M10.6 5.6a9.6 9.6 0 0 1 1.4-.1c6 0 9.5 6.5 9.5 6.5a17 17 0 0 1-2.6 3.4M6.5 7.3A16.6 16.6 0 0 0 2.5 12S6 18.5 12 18.5a9 9 0 0 0 4.3-1.1" />
      <path d="M9.9 9.9a3 3 0 0 0 4.2 4.2M4 4l16 16" />
    </>
  ),
  share: (
    <>
      <path d="M12 15V3.5M8 7l4-4 4 4" />
      <path d="M8 11H6v9.5h12V11h-2" />
    </>
  ),
  bookmark: <path d="M6.5 4h11v16.5l-5.5-4-5.5 4V4Z" />,
  chart: <path d="M4 5v14h16M8 15l3.5-4 3 2.5L19 8" />,
  star: <path d="m12 3.5 2.6 5.4 5.9.8-4.3 4.1 1 5.8-5.2-2.8-5.2 2.8 1-5.8-4.3-4.1 5.9-.8L12 3.5Z" />,
  box: (
    <>
      <path d="M3.5 8 12 3.5 20.5 8v8L12 20.5 3.5 16V8Z" />
      <path d="M3.5 8 12 12.5 20.5 8M12 12.5v8" />
    </>
  ),
  'arrow-right': <path d="M5 12h14M13 6l6 6-6 6" />,
  'arrow-up': <path d="M12 19V5M6 11l6-6 6 6" />,
  'arrow-down': <path d="M12 5v14M6 13l6 6 6-6" />,
  chevron: <path d="m9 6 6 6-6 6" />,
  'chevron-left': <path d="m15 6-6 6 6 6" />,
} satisfies Record<string, ReactNode>

export type IconName = keyof typeof PATHS

interface Props {
  name: IconName
  /** Accessible name; omit for decorative icons next to text. */
  label?: string
  className?: string
}

export function Icon({ name, label, className }: Props) {
  return (
    <svg
      className={['icon', `icon--${name}`, className].filter(Boolean).join(' ')}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      role={label ? 'img' : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : true}
    >
      {label && <title>{label}</title>}
      {PATHS[name]}
    </svg>
  )
}

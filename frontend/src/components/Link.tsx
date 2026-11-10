import type { MouseEvent, ReactNode } from 'react'
import { navigate } from '../lib/location.ts'

interface Props {
  to: string
  /** Marks the link to the screen being shown. */
  current?: boolean
  /** Accessible name, for a link that shows only an icon. */
  label?: string
  /** Called as the link is followed, before the screen changes. */
  onFollow?: () => void
  className?: string
  children: ReactNode
}

/** An in-app link: navigates without a page load, but still opens in a new tab on request. */
export function Link({ to, current, label, onFollow, className, children }: Props) {
  const open = (event: MouseEvent) => {
    if (event.metaKey || event.ctrlKey || event.shiftKey || event.button !== 0) return
    event.preventDefault()
    onFollow?.()
    navigate(to)
  }
  return (
    <a
      className={className}
      href={to}
      aria-current={current ? 'page' : undefined}
      aria-label={label}
      title={label}
      onClick={open}
    >
      {children}
    </a>
  )
}

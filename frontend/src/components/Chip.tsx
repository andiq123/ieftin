import type { ReactNode } from 'react'

interface Props {
  /** Makes the chip a toggle; leave out for a plain action. */
  selected?: boolean
  title?: string
  onClick: () => void
  children: ReactNode
}

/** A pill button, used for every filter and suggestion. */
export function Chip({ selected, title, onClick, children }: Props) {
  return (
    <button className="chip" type="button" aria-pressed={selected} title={title} onClick={onClick}>
      {children}
    </button>
  )
}

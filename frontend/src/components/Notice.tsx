import type { ReactNode } from 'react'
import { Icon, type IconName } from './Icon.tsx'

interface Props {
  icon: IconName
  title: string
  /** Announce it at once: something went wrong. */
  alert?: boolean
  action?: { label: string; onClick: () => void }
  children?: ReactNode
}

/** A centred empty or error state: symbol, a few words, and at most one way out. */
export function Notice({ icon, title, alert, action, children }: Props) {
  return (
    <div className="notice" role={alert ? 'alert' : 'status'}>
      <Icon name={icon} />
      <h2>{title}</h2>
      {children && <p>{children}</p>}
      {action && (
        <button className="button" type="button" onClick={action.onClick}>
          {action.label}
        </button>
      )}
    </div>
  )
}

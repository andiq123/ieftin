import { useState } from 'react'
import { goBack } from '../lib/location.ts'
import { ROUTES, withSearch } from '../lib/routes.ts'
import { Icon } from './Icon.tsx'

/** The navigation bar of an offer page: back to the search, and share the page. */
export function OfferBar() {
  const [copied, setCopied] = useState(false)

  const share = async () => {
    const url = window.location.href
    // The system share sheet where there is one; elsewhere the link is copied.
    if (navigator.share) return navigator.share({ url }).catch(() => undefined)
    await navigator.clipboard.writeText(url)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <>
      <button className="bar-button" type="button" onClick={() => goBack(withSearch(ROUTES.search))}>
        <Icon name="chevron-left" />
        Search
      </button>
      <button
        className="bar-button navbar__end"
        type="button"
        aria-label={copied ? 'Link copied' : 'Share'}
        title={copied ? 'Link copied' : 'Share'}
        onClick={share}
      >
        <Icon name={copied ? 'check' : 'share'} />
      </button>
    </>
  )
}

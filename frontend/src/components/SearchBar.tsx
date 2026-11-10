import { type FormEvent, useState } from 'react'
import { Icon } from './Icon.tsx'

interface Props {
  query: string
  onSearch: (query: string) => void
}

/** Searches on Enter or the button: each search asks the stores, so not on every keystroke. */
export function SearchBar({ query, onSearch }: Props) {
  const [draft, setDraft] = useState(query)
  // Follow outside changes such as back/forward or an example search.
  const [syncedQuery, setSyncedQuery] = useState(query)
  if (syncedQuery !== query) {
    setSyncedQuery(query)
    setDraft(query)
  }

  const submit = (event: FormEvent) => {
    event.preventDefault()
    if (draft.trim() !== query.trim()) onSearch(draft.trim())
  }

  return (
    <form className="search" role="search" onSubmit={submit}>
      <Icon name="search" />
      <input
        type="search"
        aria-label="Search for a product"
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
        placeholder="iPhone 15 Pro, PlayStation 5…"
        autoComplete="off"
        enterKeyHint="search"
        autoFocus
      />
      <button type="submit" aria-label="Search">
        <Icon name="arrow-right" />
      </button>
    </form>
  )
}

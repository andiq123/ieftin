import type { CSSProperties, ReactNode } from 'react'

interface Option<Value extends string> {
  value: Value
  label: ReactNode
}

interface Props<Value extends string> {
  /** Accessible name of the group. */
  label: string
  options: readonly Option<Value>[]
  value: Value
  /** Also called when the current option is chosen again. */
  onChange: (value: Value) => void
}

/** A segmented control: one choice out of a few, with a thumb that slides to it. */
export function Segmented<Value extends string>({ label, options, value, onChange }: Props<Value>) {
  const position = {
    '--count': options.length,
    '--index': options.findIndex((option) => option.value === value),
  } as CSSProperties
  return (
    <div className="segmented" role="group" aria-label={label} style={position}>
      <span className="segmented__thumb" aria-hidden="true" />
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          aria-pressed={option.value === value}
          onClick={() => onChange(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  )
}

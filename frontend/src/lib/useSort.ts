import { useMemo, useState } from 'react'
import { withTransition } from './transition.ts'

export type Direction = 'ascending' | 'descending'

type Value = string | number | null

/** How each sortable column reads its value from a row. */
export type Columns<Row, Key extends string> = Record<Key, (row: Row) => Value>

function compare(a: Value, b: Value): number {
  if (a === null || b === null) return a === b ? 0 : a === null ? 1 : -1
  return typeof a === 'number' && typeof b === 'number' ? a - b : String(a).localeCompare(String(b), 'en')
}

/** Client-side table sorting: choosing the active column again flips the direction. */
export function useSort<Row, Key extends string>(
  rows: Row[],
  columns: Columns<Row, Key>,
  initial: { key: Key; direction: Direction },
) {
  const [sort, setSort] = useState(initial)

  const sorted = useMemo(() => {
    const sign = sort.direction === 'ascending' ? 1 : -1
    const read = columns[sort.key]
    return [...rows].sort((a, b) => {
      const [x, y] = [read(a), read(b)]
      // Missing values stay at the bottom in either direction.
      return x === null || y === null ? compare(x, y) : sign * compare(x, y)
    })
  }, [rows, columns, sort])

  const sortBy = (key: Key) =>
    withTransition('reorder', () =>
      setSort((current) => ({
        key,
        direction: current.key === key && current.direction === 'ascending' ? 'descending' : 'ascending',
      })),
    )

  return { sorted, sort, sortBy }
}

import { withTx } from './tx'
import { enqueue } from '@sample/jobs'

export function rebound(rows: { id: string }[]) {
  let work = (_row: { id: string }) => 0
  work = (row) => enqueue({ id: row.id })
  return withTx(async () => {
    return rows.map(work)
  })
}

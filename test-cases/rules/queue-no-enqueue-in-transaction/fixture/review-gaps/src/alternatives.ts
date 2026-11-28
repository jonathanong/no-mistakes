import { withTx } from './tx'
import { enqueue } from '@sample/jobs'

const flag = true

export function choose(rows: { id: string }[]) {
  return withTx(async () => {
    return (flag ? Array.from : Array.of)(rows, (row: { id: string }) => enqueue({ id: row.id }))
  })
}

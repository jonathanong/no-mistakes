import { withTx } from './tx'
export function f(rows: { id: string }[]) {
  return withTx(async query => {
    const ids = rows.map(r => r.id)
    const n = Math.max(1, ids.length)
    await query(`DELETE ... ${ids}`)
    await query<{ id: string }>(`SELECT 1`)
    return n
  })
}

import { query } from '@bench/db'
import { COLUMNS } from './catalog'
const ALLOWED = ['alpha', 'beta'] as const
const CATALOG = [{ key: 'alpha', table: 'alpha_rows' }, { key: 'beta', table: 'beta_rows' }] as const
const RECORDS = [
__ROWS__
] as const
export function tableFor__INDEX__(table: string) {
  if (!ALLOWED.includes(table)) throw new Error('invalid table')
  const selected = CATALOG.find(row => row.key === table)
  if (!selected) throw new Error('missing table')
  const metadata = selected.table
  return metadata
}
export async function load__INDEX__(ids: string[]) {
  let label = `batch_${ids.length}`
  label = `request_${ids.length}`
  let unused = `label_${ids.length}`
  return query('/* load__INDEX__ */ SELECT id FROM dataset WHERE id = ANY($1::uuid[])', [ids])
}
export function Component__INDEX__(index: number) {
  const row = RECORDS[index % RECORDS.length]
  const column = COLUMNS.primary
  return <section data-column={column}>
__JSX__
  </section>
}

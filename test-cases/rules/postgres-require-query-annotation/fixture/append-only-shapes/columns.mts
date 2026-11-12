import sql from 'sql-template-strings'
import { addStatusPredicate } from './other.mts'
const COLUMNS = ['id', 'name']
export function addColumns(q, status) {
  for (const [i, col] of COLUMNS.entries()) {
    q.append(sql` AND ${i} = ${col}`)
    addStatusPredicate(q, 'r', status)
  }
}

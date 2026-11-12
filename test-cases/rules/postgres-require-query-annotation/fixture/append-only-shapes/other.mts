import sql from 'sql-template-strings'
export function addStatusPredicate(q, table, status) {
  q.append(sql` AND ${table} = ${status}`)
}

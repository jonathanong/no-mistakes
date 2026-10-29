import sql from 'sql-template-strings'
export function filters(query, { enabled }) {
  if (enabled) query.append(sql` WHERE enabled = ${enabled}`)
  pagination(query, { enabled })
}
function pagination(query, { enabled }) {
  if (enabled) query.append(sql` LIMIT ${1}`)
}
export function order(query, limit) {
  query.append(suffix(limit))
}
export function unsafe(query) { expose(query) }
export function cycle(query) { other(query) }
function other(query) { cycle(query) }

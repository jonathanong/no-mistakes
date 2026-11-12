import sql from 'sql-template-strings'
export function appendDescendantsCte(q, options = {}) {
  if (options.enabled) q.append(sql` WITH RECURSIVE d AS (SELECT 1)`)
  for (const id of options.ids ?? []) q.append(sql` AND d.id = ${id}`)
}

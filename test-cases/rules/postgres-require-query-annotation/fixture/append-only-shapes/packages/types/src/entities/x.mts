import sql from 'sql-template-strings'
export function addCte(q: ReturnType<typeof sql>, o = {}) {
  if (o.enabled) q.append(sql` TRUE`)
  const ids = o.ids || []
  ids.forEach((id, i) => {
    if (i > 0) q.append(sql`, `)
    q.append(sql`${id}::uuid`)
  })
}

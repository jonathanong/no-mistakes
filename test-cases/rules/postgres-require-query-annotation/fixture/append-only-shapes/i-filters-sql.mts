import sql from 'sql-template-strings'
export function appendFilters(query, filters) {
  if (filters.length === 0) return
  query.append(sql` WHERE `)
  filters.forEach((f, i) => {
    if (i > 0) query.append(sql` AND `)
    query.append(f)
  })
}

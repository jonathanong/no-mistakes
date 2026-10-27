import sql from 'sql-template-strings'

export function importedQuery() {
  const query = sql`SELECT a.id FROM accounts a
    JOIN LATERAL (
      SELECT * FROM accounts imported_first
      WHERE imported_first.id = a.id
    ) f ON true`
  query.append(sql` UNION ALL SELECT a.id FROM accounts a
    JOIN LATERAL (
      SELECT * FROM accounts imported_second
      WHERE imported_second.id = a.id
    ) s ON true`)
  return query
}

export function directImportedQuery() {
  return sql`SELECT a.id FROM accounts a
    JOIN LATERAL (
      SELECT * FROM accounts imported_direct
      WHERE imported_direct.id = a.id
    ) d ON true`
}

// The exported declaration alone must not count as executed SQL.
export function unusedImportedQuery() {
  return sql`SELECT * FROM accounts unused_imported`
}

export function suppressedImportedQuery() {
  // no-mistakes-disable-next-line postgres-explicit-columns
  return sql`SELECT * FROM accounts suppressed`
}

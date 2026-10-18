// A SQL-looking call on an unproven receiver needs a diagnostic, not a verdict.
export function mixed(unknownDb: unknown, http: { query(value: object): void }) {
  ;(unknownDb as { query(value: string): void }).query('SELECT id FROM unproven_sql')
  http.query({ resource: 'http' })
}

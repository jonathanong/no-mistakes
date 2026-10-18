// A SQL-looking call on an unproven receiver needs a diagnostic, not a verdict.
export function mixed(unknownDb: unknown, http: { query(value: object): void }) {
  ;(unknownDb as { query(value: string): void }).query('SELECT id FROM unproven_sql')
  ;(unknownDb as { query(value: string): void }).query('WITH rows AS (SELECT 1) SELECT * FROM rows')
  ;(unknownDb as { query(value: string): void }).query('BEGIN')
  ;(unknownDb as { query(value: string): void }).query('CREATE TABLE diagnostic_only (id int)')
  http.query({ resource: 'http' })
}

import sql from 'sql-template-strings'

export function generatedCursor(after: string) { // The marker-shaped projection is a real column.
  return sql`SELECT sql_placeholder_1 FROM orders WHERE id > ${after} ORDER BY id LIMIT 10`
}

export function literalCursor(after: string) { // Same SQL text, but the marker-shaped WHERE column is literal.
  return sql`SELECT ${after} FROM orders WHERE id > sql_placeholder_1 ORDER BY id LIMIT 10`
}

export function wrappedCursor(after: string) { // Emoji protects parser column offsets in multiline SQL.
  return sql`AND EXISTS (
    SELECT '🚀' AS label, sql_placeholder_1 FROM orders WHERE id > ${after} ORDER BY id LIMIT 10
  )`
}

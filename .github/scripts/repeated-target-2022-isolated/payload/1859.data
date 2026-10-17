import sql from 'sql-template-strings'

export function makeQuery() {
  return sql`/* shadowed helper */ SELECT 1`
}

export function appendKnown(query: { append: (sql: string) => void }, _unused?: unknown): void {
  query.append(' /* known suffix */')
}

export async function appendLater(query: { append: (sql: string) => void }): Promise<void> {
  await Promise.resolve()
  query.append(' UPDATE posts SET title = title')
}

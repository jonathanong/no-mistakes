import sql from "sql-template-strings";

export function importedStatement(): { text: string; values: unknown[] } {
  return sql`SELECT * FROM items WHERE id = ${1}`;
}

export function importedWithOptions(options: { limit: number; marker: string }): { text: string; values: unknown[] } {
  return sql`SELECT * FROM items WHERE marker = ${options.marker} LIMIT ${options.limit}`;
}

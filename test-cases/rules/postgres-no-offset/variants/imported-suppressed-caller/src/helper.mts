export function pageClause(size: string): string {
  return `ORDER BY id LIMIT ${size} OFFSET 40`;
}

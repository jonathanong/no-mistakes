export function pageClause(orderColumn: string, size: string): string {
  return `ORDER BY ${orderColumn}
    LIMIT ${size} OFFSET 40`;
}

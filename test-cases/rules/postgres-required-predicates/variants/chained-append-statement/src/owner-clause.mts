export function ownerClause(column: string): string {
  return `(${column} IS NULL)`;
}

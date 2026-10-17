export function lateralRows(table: string): string {
  return `JOIN LATERAL (
    SELECT * FROM ${table} sub
    WHERE sub.id = a.id
  ) l ON true`;
}

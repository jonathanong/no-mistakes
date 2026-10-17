const view = "view_active_plans";

export function concatenated(column: string): string {
  return column + ` IS NULL OR EXISTS (SELECT 1 FROM ${view} p WHERE p.account_id = 1)`;
}

export function transformed(column: string): string {
  return `${column.toUpperCase()} IS NULL OR EXISTS (SELECT 1 FROM view_active_plans p WHERE p.account_id = 1)`;
}

export function reused(column: string): string {
  const copy = column;
  return `${copy} IS NULL OR EXISTS (SELECT 1 FROM view_active_plans p WHERE p.account_id = 1)`;
}

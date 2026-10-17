export function ownerVisibleSql(parentCol: string, ownerCol: string): string {
  return `(${parentCol} IS NULL OR
    EXISTS (SELECT 1 FROM view_active_plans p WHERE p.account_id = ${ownerCol}))`;
}

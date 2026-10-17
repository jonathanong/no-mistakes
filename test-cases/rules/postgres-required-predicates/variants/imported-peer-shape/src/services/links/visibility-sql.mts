export function ownerVisibleSql(parentCol: string, ownerCol: string): string {
  return ` AND (${parentCol} IS NULL OR EXISTS (SELECT 1 FROM view_active_plans p2 WHERE p2.account_id = ${ownerCol}))`;
}

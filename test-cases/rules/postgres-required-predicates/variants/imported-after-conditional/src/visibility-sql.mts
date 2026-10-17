export function ownerVisibleSql(parentCol: string, ownerCol: string): string {
  return ` AND (${parentCol} IS NULL OR EXISTS (SELECT 1 FROM view_active_plans h WHERE h.account_id = ${ownerCol}))`;
}

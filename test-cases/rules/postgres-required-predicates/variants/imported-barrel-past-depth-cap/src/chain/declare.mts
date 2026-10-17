export function visibleSql(ownerCol: string): string {
  return `EXISTS (SELECT 1 FROM view_active_plans p WHERE p.account_id = ${ownerCol})`;
}

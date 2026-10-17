export function visibleSql(ownerCol: string): string {
  return `EXISTS (SELECT 1 FROM view_active_plans b WHERE b.account_id = ${ownerCol})`;
}

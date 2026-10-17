export function precedence(ownerCol: string): string {
  return `EXISTS (SELECT 1 FROM view_active_plans e WHERE e.account_id = ${ownerCol})`;
}

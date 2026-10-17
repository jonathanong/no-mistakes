export function reexported(ownerCol: string): string {
  return `EXISTS (SELECT 1 FROM view_active_plans r WHERE r.account_id = ${ownerCol})`;
}

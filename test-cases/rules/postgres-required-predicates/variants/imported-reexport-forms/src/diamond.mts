export function viaDiamond(ownerCol: string): string {
  return `EXISTS (SELECT 1 FROM view_active_plans d WHERE d.account_id = ${ownerCol})`;
}

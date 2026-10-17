export default function visibleByDefault(ownerCol: string): string {
  return `EXISTS (SELECT 1 FROM view_active_plans x WHERE x.account_id = ${ownerCol})`;
}

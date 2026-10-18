export function ownerVisible(parent: string, owner: string): string {
  return `(${parent} IS NULL OR EXISTS (SELECT 1 FROM view_active_plans p WHERE p.account_id = ${owner}))`;
}

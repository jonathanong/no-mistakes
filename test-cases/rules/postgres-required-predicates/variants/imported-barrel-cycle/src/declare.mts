// Reachable through the cycle's second barrel, but the cycle itself is what
// the fixture protects: a cyclic walk must fail closed, not pick this up.
export function visibleSql(ownerCol: string): string {
  return `EXISTS (SELECT 1 FROM view_active_plans p WHERE p.account_id = ${ownerCol})`;
}

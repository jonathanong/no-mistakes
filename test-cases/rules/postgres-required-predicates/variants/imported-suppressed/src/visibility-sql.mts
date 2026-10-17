export function reviewedSql(ownerCol: string): string {
  // no-mistakes-disable-next-line postgres-required-predicates
  return `EXISTS (SELECT 1 FROM view_active_plans p WHERE p.account_id = ${ownerCol})`;
}

export function unreviewedSql(ownerCol: string): string {
  return `EXISTS (SELECT 1 FROM view_active_plans p WHERE p.account_id = ${ownerCol})`;
}

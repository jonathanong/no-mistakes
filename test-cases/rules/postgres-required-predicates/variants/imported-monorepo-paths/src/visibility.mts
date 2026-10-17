// The root's `@/*` alias. The package below maps the same alias to its own
// directory, so a caller in that package must never resolve here.


export function ownerVisibleSql(ownerCol: string): string {
  return `EXISTS (SELECT 1 FROM view_active_plans root_decoy WHERE root_decoy.account_id = ${ownerCol})`;
}

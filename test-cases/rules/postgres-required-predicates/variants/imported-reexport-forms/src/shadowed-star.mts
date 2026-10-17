// `index.mts` re-exports `precedence` explicitly, which shadows this star export.
export function precedence(ownerCol: string): string {
  return `EXISTS (SELECT 1 FROM view_active_plans s WHERE s.account_id = ${ownerCol})`;
}

import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";

export function withTierFilter(query: SQLStatement, tier: string): SQLStatement {
  query.append(sql` AND p.account_id IN (SELECT z.account_id FROM view_active_plans z WHERE z.tier = ${tier})`);
  return query;
}

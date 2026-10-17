import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";

export function buildEligibilityFilter(alias: string, root: string): SQLStatement {
  return sql` AND EXISTS (SELECT 1 FROM view_active_plans q WHERE q.tier = 'b')`;
}

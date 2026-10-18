import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";

export function appendTier(query: SQLStatement, strict: boolean) {
  if (strict) {
    query.append(sql` AND EXISTS (SELECT 1 FROM view_active_plans s WHERE s.tier = 'strict')`);
  } else {
    query.append(sql` AND EXISTS (SELECT 1 FROM view_active_plans l WHERE l.tier = 'loose')`);
  }
}

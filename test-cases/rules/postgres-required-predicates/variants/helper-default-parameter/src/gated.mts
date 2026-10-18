import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";

export function gated(alias: string, requireLive = true): SQLStatement {
  if (requireLive) {
    return sql`SELECT 1 FROM view_active_plans a WHERE a.tier = 'live'`.append(` /* ${alias} */`);
  }
  return sql`SELECT 1 FROM view_active_plans b WHERE b.tier = 'any'`.append(` /* ${alias} */`);
}

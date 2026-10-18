import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";

export function appendListQuery(query: SQLStatement, options: { all: boolean; limit: number }) {
  if (options.all) {
    query.append(sql` AND p.tier IN (SELECT z.tier FROM view_active_plans z WHERE z.tier = 'a')`);
    return;
  }
  query.append(sql` LIMIT ${options.limit}`);
}

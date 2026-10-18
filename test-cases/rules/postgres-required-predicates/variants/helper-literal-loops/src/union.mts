import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";

// A loop over a parameter is unrolled only when the call site passes an
// array literal of at most sixteen known elements.
export function unionOf(tables: string[] = []): SQLStatement {
  const statement = sql`SELECT 1 FROM view_active_plans v WHERE v.tier = 'a'`;
  for (const table of tables) {
    statement.append(sql` UNION ALL SELECT 1 FROM `).append(table).append(sql` t WHERE t.tier = 'a'`);
  }
  return statement;
}

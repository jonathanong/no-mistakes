import { query, sql } from 'db';
import { statementSql } from './statement-fragment';
export function run(flag: boolean) {
  return query(
    (flag ? sql``.append(statementSql()).append(sql`CROSS JOIN orders o;`) : sql``)
      .append(statementSql())
      .append(sql`CROSS JOIN other t`)
  );
}

import { query, sql } from 'db';
function statementSql() { return sql`SELECT a.id FROM accounts a `; }
export function run(flag: boolean) {
  return query(
    statementSql().append(sql`CROSS JOIN orders o;`)
      .append(statementSql())
      .append(sql`CROSS JOIN other t`)
  );
}

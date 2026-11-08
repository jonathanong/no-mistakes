import { query, sql } from 'db';
function statementSql() { return sql`SELECT ${1} AS value, a.id FROM accounts a `; }
export function run(flag: boolean) {
  return query(
    (flag ? sql`\n`.append(statementSql()).append(sql`CROSS JOIN orders o;`) : sql``)
      .append(statementSql())
      .append(sql`CROSS JOIN other t`)
  );
}

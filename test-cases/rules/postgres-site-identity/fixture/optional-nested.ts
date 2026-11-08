import { query, sql } from 'db';
function statementSql() { return sql`SELECT a.id FROM accounts a `; }
function build(flag: boolean) {
  return (flag ? statementSql().append(sql`CROSS JOIN orders o;`) : sql``)
      .append(statementSql())
      .append(sql`CROSS JOIN other t`);
}
export function run(flag: boolean) { return query(build(flag)); }

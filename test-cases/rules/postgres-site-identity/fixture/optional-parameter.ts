import { query } from 'db';
import sql, { type SQLStatement } from 'sql-template-strings';
function statementSql(value: SQLStatement) {
  return value.append(sql`SELECT a.id FROM accounts a `);
}
export function run(flag: boolean) {
  return query(
    (flag ? statementSql(sql``).append(sql`CROSS JOIN orders o;`) : sql``)
      .append(statementSql(sql``))
      .append(sql`CROSS JOIN other t`)
  );
}

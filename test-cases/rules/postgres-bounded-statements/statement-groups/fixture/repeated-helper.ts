import { query, sql } from '@example/db';

function statementSql() {
  return sql`SELECT i.id FROM items i `;
}

export function run(flag: boolean) {
  return query(
    (flag ? sql`\n` : sql``)
      .append(statementSql())
      .append(sql`;`)
      .append(statementSql())
      .append(sql`CROSS JOIN feeds f`)
  );
}

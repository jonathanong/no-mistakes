import { query, sql } from '@example/db';

function bodySql() {
  return sql`SELECT i.id FROM items i CROSS JOIN `;
}
function buildSql(flag: boolean) {
  return flag
    ? sql`\n`.append(bodySql()).append(sql`feeds f`)
    : bodySql().append(sql`topics t`);
}

export function run(flag: boolean) {
  return query(buildSql(flag));
}

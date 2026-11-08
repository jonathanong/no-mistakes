import { query, sql } from 'db';
function bodySql() { return sql`SELECT a.id FROM accounts a CROSS JOIN `; }
function build(flag: boolean) {
  return flag ? sql`\n`.append(bodySql()).append(sql`orders o`)
    : bodySql().append(sql`other t`);
}
export function run(flag: boolean) { return query(build(flag)); }

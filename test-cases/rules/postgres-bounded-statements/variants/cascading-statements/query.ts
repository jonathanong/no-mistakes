import { query, sql } from "@example/db";

export function run(flag: boolean) {
  const statement = sql`SELECT a.id FROM accounts a`;
  if (flag) {
    statement.append(sql` JOIN orders o ON o.account_id = a.id`);
  } else {
    statement.append(sql` JOIN invoices i ON i.id = a.id`);
  }
  return query(statement);
}

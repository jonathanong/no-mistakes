import { query, sql } from "@example/db";

const archived = sql`AND archived_at IS NOT NULL`;

export function load(accountId: string) {
  return query(
    sql`SELECT id FROM documents WHERE account_id = ${accountId}`.append(sql` ${archived}`),
  );
}

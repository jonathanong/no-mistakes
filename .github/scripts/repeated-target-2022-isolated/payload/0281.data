import { query, sql } from "@example/db";

const notDeleted = sql`AND deleted_at IS NULL`;
let filter = sql``;

export function load(accountId: string, archived: boolean) {
  // `active` aliases a fragment binding through a conditional.
  const active = archived ? filter : notDeleted;
  filter = sql`AND archived_at IS NOT NULL`;
  return query(sql`SELECT id FROM documents WHERE account_id = ${accountId} ${active}`);
}

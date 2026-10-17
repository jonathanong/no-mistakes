import sql from "sql-template-strings";
import { read } from "@example/db";
import { feedVisibleSql } from "@modules/feed-query-builders";

// Resolved through the tsconfig `paths` alias, not a relative import.
export async function feed(accountId: string) {
  const query = sql`SELECT f.id FROM feed f WHERE f.account_id = ${accountId} AND `;
  query.append(feedVisibleSql("f.owner_id"));
  return read(query);
}

import { query } from "@example/db";
import { sql } from "@example/dbx";

export function load(accountId: string) {
  return query(sql`
    SELECT id
    FROM documents
    WHERE account_id = ${accountId}
  `);
}

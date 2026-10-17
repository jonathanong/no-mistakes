import { query } from "@example/db";
import { type sql as dbSql } from "@example/db";

export function load(accountId: string) {
  return query(dbSql`
    SELECT id
    FROM documents
    WHERE account_id = ${accountId}
  `);
}

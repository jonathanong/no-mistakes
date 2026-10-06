import { query, sql as dbSql } from "@example/db";

// The inner binding rebinds the trusted local. Fail closed, same as a shadowed sql.
export function load(accountId: string) {
  const dbSql = (strings: TemplateStringsArray) => strings;
  return query(dbSql`
    SELECT id
    FROM documents
    WHERE account_id = ${accountId}
  `);
}

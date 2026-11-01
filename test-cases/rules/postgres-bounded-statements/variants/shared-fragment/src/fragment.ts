import { sql } from "@example/db";

export function sharedFragmentSql() {
  return sql` JOIN orders o ON o.account_id = a.id`;
}

import { sql } from "@example/db";

export function singleRootFragmentSql() {
  return sql` FROM invoices`;
}

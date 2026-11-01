import { sql } from "@example/db";

export function pairFragmentSql() {
  return sql` orders o JOIN invoices i ON i.id = o.id`;
}

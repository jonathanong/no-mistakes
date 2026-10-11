import { query, sql } from "@example/db";
const statement = sql`SELECT id FROM accounts`;
// The second matching label runs both appends, making the combined lock unsafe.
switch (runtimeMode) {
  case (statement.append(sql` WHERE id IN ($1, $2)`), "first"): break;
  case (statement.append(sql` FOR UPDATE`), "second"): query(statement); break; // findings: unanalyzable
}
query(flag ? statement : sql`SELECT 1`); // findings: unanalyzable

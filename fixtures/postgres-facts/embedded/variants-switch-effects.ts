import { query, sql } from "@example/db";
const unrelated = sql`SELECT 99`;
const statement = sql`SELECT id FROM accounts`;
// The second matching label runs both appends, making the combined lock unsafe.
switch (runtimeMode) {
  case (statement.append(sql` WHERE id IN ($1, $2)`), "first"): break;
  case (statement.append(sql` FOR UPDATE`), "second"): query(statement); break; // findings: unanalyzable
}
query(flag ? statement : sql`SELECT 1`); // findings: unanalyzable
const failedLabel = sql`SELECT id FROM users`;
switch (runtimeMode) {
  case (failedLabel.append(sql` OFFSET 1`), "first"): break;
  case "second": query(failedLabel); break;
}
const discriminant = sql`SELECT id FROM users`;
switch ((discriminant.append(sql` WHERE id IN ($1, $2)`), runtimeMode)) {
  case "lock": discriminant.append(sql` FOR UPDATE`); query(flag ? discriminant : sql`SELECT 2`); break;
  default: break;
}
query(flag ? discriminant : sql`SELECT 3`);
let staticLabels = "SELECT 4";
switch (runtimeMode) {
  case "first": staticLabels += "0"; break;
  default: staticLabels += "1";
}
query(staticLabels);
let assigned = "SELECT id FROM users";
switch (runtimeMode) {
  case (assigned += " OFFSET 2", "first"): break;
  case "second": query(assigned); break;
}
query(unrelated);

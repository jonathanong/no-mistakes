import { query } from "@example/db";
query(flag
  ? "SELECT id FROM accounts OFFSET 1"
  : "SELECT id FROM accounts LIMIT 1");

// Exact and exhaustive-union evidence must agree on the proven violation.
query("SELECT id FROM accounts OFFSET 1");
// The opaque call must not hide the separate proven violations above.
declare const runtimeSql: string;
query(runtimeSql);

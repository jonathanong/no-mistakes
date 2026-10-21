import { query } from "@example/db";
query(flag
  ? "SELECT * FROM accounts"
  : "SELECT id FROM accounts");

// Exact and exhaustive-union evidence must agree on the proven violation.
query("SELECT * FROM accounts");
// The opaque call must not hide the separate proven violations above.
declare const runtimeSql: string;
query(runtimeSql);

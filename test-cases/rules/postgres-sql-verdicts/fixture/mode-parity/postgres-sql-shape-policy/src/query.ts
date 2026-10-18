import { query } from "@example/db";
query(flag
  ? "SELECT pg_sleep(1)"
  : "SELECT 1");

// Exact and exhaustive-union evidence must agree on the proven violation.
query("SELECT pg_sleep(1)");
// The opaque call must not hide the separate proven violations above.
declare const runtimeSql: string;
query(runtimeSql);

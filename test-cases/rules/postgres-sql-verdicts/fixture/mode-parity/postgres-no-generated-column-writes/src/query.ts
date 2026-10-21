import { query } from "@example/db";
query(flag
  ? "UPDATE items SET created_at = now() WHERE id = $1"
  : "UPDATE items SET note = $2 WHERE id = $1");

// Exact and exhaustive-union evidence must agree on the proven violation.
query("UPDATE items SET created_at = now() WHERE id = $1");
// The opaque call must not hide the separate proven violations above.
declare const runtimeSql: string;
query(runtimeSql);

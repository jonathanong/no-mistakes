import { query, sql } from "@example/db";
query(flag ? "SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE" : "SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE SKIP LOCKED");

// Exact and exhaustive-union evidence must agree on the proven violation.
query("SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE");
// The opaque call must not hide the separate proven violations above.
declare const runtimeSql: string;
query(runtimeSql);

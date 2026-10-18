import { query, sql } from "@example/db";
query(flag ? "SELECT id FROM accounts" : "SELECT id FROM accounts WHERE active IS TRUE ORDER BY id");

// Exact and exhaustive-union evidence must agree on the proven violation.
query("SELECT id FROM accounts");
// The opaque call must not hide the separate proven violations above.
declare const runtimeSql: string;
query(runtimeSql);

import { query } from "@example/db";
query(flag
  ? "CREATE INDEX accounts_email ON accounts (email)"
  : "SELECT 1");

// Exact and exhaustive-union evidence must agree on the proven violation.
query("CREATE INDEX accounts_email ON accounts (email)");
// The opaque call must not hide the separate proven violations above.
declare const runtimeSql: string;
query(runtimeSql);

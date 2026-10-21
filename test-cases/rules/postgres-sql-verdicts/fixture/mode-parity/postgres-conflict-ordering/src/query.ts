import { query } from "@example/db";
query(flag
  ? "INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ON CONFLICT (id) DO NOTHING"
  : "INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ORDER BY input.id ON CONFLICT (id) DO NOTHING");

// Exact and exhaustive-union evidence must agree on the proven violation.
query("INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ON CONFLICT (id) DO NOTHING");
// The opaque call must not hide the separate proven violations above.
declare const runtimeSql: string;
query(runtimeSql);

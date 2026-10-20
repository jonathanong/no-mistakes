import { query } from "@example/db";
// Exact SELECTs need no conflict-policy statement extraction.
query("SELECT id FROM accounts");
// Conservative substring matching may retain quoted INSERT text; it must not drop a real INSERT.
query("SELECT 'INSERT' AS label");
query("WITH c AS (InSeRt INTO t VALUES (1) ON CONFLICT DO NOTHING) SELECT 1");
query("SELECT id FROM accounts ORDER BY id");
// Malformed non-INSERT SQL keeps the existing conflict-policy result in every mode.
query("SELECT (");
// WITH alone does not imply an INSERT; keep missing facts distinct from missing conflicts.
query("WITH v AS (SELECT id FROM accounts) SELECT id FROM v");
query("WITH (");

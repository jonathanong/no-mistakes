import { query } from "@example/db";
// Exact SELECTs need no conflict-policy statement extraction.
query("SELECT id FROM accounts");
// Conservative substring matching may retain quoted INSERT text; it must not drop a real INSERT.
query("SELECT 'INSERT' AS label");
query(
  "WITH changed AS (InSeRt INTO accounts (id) SELECT id FROM accounts ON CONFLICT (id) DO NOTHING RETURNING id) SELECT id FROM changed",
);
query("SELECT id FROM accounts ORDER BY id");
// Malformed non-INSERT SQL keeps the existing conflict-policy result in every mode.
query("SELECT (");

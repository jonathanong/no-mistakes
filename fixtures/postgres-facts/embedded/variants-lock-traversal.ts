import { query } from "@example/db";
export function run(flag: boolean) {
  query(flag
    ? "SELECT id FROM accounts WHERE id = ANY($1) ORDER BY id FOR UPDATE"
    : "SELECT id FROM accounts WHERE id = ANY($1) FOR UPDATE");
  // Projection subqueries and INSERT sources retain the existing lock traversal boundary.
  query(flag
    ? "SELECT (SELECT id FROM accounts FOR UPDATE) FROM pending WHERE id = 1"
    : "SELECT (SELECT id FROM accounts FOR UPDATE) FROM pending WHERE id = 2");
  query(flag
    ? "INSERT INTO pending (id) SELECT id FROM accounts ORDER BY id FOR UPDATE"
    : "INSERT INTO pending (id) SELECT id FROM accounts FOR UPDATE");
  query(flag
    ? "SELECT id FROM pending WHERE id IN (SELECT id FROM accounts ORDER BY id FOR UPDATE)"
    : "SELECT id FROM pending WHERE id IN (SELECT id FROM accounts FOR UPDATE)");
  query(flag
    ? "WITH locked AS (SELECT id FROM accounts ORDER BY id FOR UPDATE) SELECT id FROM locked"
    : "WITH locked AS (SELECT id FROM accounts FOR UPDATE) SELECT id FROM locked");
}

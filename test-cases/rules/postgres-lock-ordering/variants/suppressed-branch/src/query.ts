import { query } from "@example/db";
query(flag
  // no-mistakes-disable-next-line postgres-lock-ordering
  ? "SELECT id FROM accounts WHERE id IN (1, 2) FOR UPDATE"
  : "SELECT id FROM accounts WHERE id IN (1, 2) ORDER BY id FOR UPDATE");

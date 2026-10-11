import { query, sql } from "@example/db";
// no-mistakes-disable-next-line postgres-lock-ordering
query(flag
  ? "SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE"
  : "SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE SKIP LOCKED");

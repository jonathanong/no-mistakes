import { query, sql } from "@example/db";
// no-mistakes-disable-next-line postgres-bounded-statements
query(flag
  ? "SELECT id FROM accounts"
  : "SELECT id FROM accounts LIMIT 2");

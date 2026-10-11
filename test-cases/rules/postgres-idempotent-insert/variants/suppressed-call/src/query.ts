import { query } from "@example/db";
// no-mistakes-disable-next-line postgres-idempotent-insert
query(flag
  ? "INSERT INTO items (id) VALUES (1)"
  : "INSERT INTO items (id) VALUES (1) ON CONFLICT DO NOTHING");

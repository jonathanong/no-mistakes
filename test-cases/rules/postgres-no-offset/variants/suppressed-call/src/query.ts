import { query } from "@example/db";
// no-mistakes-disable-next-line postgres-no-offset
query(flag
  ? "SELECT id FROM accounts OFFSET 1"
  : "SELECT id FROM accounts LIMIT 1");

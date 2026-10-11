import { query } from "@example/db";
// no-mistakes-disable-next-line postgres-explicit-columns
query(flag
  ? "SELECT * FROM accounts"
  : "SELECT id FROM accounts");

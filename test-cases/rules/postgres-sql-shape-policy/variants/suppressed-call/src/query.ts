import { query } from "@example/db";
// no-mistakes-disable-next-line postgres-sql-shape-policy
query(flag
  ? "SELECT pg_sleep(1)"
  : "SELECT 1");

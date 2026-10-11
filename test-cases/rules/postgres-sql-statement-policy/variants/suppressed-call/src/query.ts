import { query } from "@example/db";
// no-mistakes-disable-next-line postgres-sql-statement-policy
query(flag
  ? "CREATE INDEX accounts_email ON accounts (email)"
  : "SELECT 1");

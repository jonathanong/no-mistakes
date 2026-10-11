import { query } from "@example/db";
query(flag
  ? "CREATE INDEX accounts_email ON accounts (email)"
  : "SELECT 1");

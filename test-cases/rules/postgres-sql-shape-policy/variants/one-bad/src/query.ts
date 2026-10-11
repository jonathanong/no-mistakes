import { query } from "@example/db";
query(flag
  ? "SELECT pg_sleep(1)"
  : "SELECT 1");

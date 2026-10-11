import { query } from "@example/db";
query(flag
  ? "SELECT id FROM accounts LIMIT 2"
  : "SELECT id FROM accounts LIMIT 1");

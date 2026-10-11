import { query } from "@example/db";
query(flag
  ? "SELECT id FROM orders WHERE created_at > $1"
  : "SELECT id FROM orders WHERE id > $1");

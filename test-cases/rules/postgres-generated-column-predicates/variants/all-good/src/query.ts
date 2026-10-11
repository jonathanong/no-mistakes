import { query } from "@example/db";
query(flag
  ? "SELECT id FROM orders WHERE id > $2"
  : "SELECT id FROM orders WHERE id > $1");

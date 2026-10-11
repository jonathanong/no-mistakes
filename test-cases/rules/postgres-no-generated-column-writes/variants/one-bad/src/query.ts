import { query } from "@example/db";
query(flag
  ? "UPDATE items SET created_at = now() WHERE id = $1"
  : "UPDATE items SET note = $2 WHERE id = $1");

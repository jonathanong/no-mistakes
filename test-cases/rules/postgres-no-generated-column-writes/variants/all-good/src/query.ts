import { query } from "@example/db";
query(flag
  ? "UPDATE items SET note = $3 WHERE id = $1"
  : "UPDATE items SET note = $2 WHERE id = $1");

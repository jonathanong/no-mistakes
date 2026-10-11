import { query } from "@example/db";
query(flag
  ? "INSERT INTO items (id) VALUES (1)"
  : "INSERT INTO items (id) VALUES (1) ON CONFLICT DO NOTHING");

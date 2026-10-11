import { query } from "@example/db";
// no-mistakes-disable-next-line postgres-generated-column-predicates
query(flag
  ? "SELECT id FROM orders WHERE created_at > $1"
  : "SELECT id FROM orders WHERE id > $1");

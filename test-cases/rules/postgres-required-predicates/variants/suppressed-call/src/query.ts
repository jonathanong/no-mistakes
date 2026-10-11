import { query, sql } from "@example/db";
// no-mistakes-disable-next-line postgres-required-predicates
query(flag
  ? "SELECT id FROM accounts"
  : "SELECT id FROM accounts WHERE active IS TRUE ORDER BY id");

import { query, sql } from "@example/db";
query(flag ? "SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE" : "SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE SKIP LOCKED");

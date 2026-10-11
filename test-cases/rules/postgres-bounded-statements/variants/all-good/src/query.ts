import { query, sql } from "@example/db";
query(flag ? "SELECT id FROM accounts LIMIT 1" : "SELECT id FROM accounts LIMIT 2");

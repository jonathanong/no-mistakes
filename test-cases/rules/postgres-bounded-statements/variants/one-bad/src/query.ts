import { query, sql } from "@example/db";
query(flag ? "SELECT id FROM accounts" : "SELECT id FROM accounts LIMIT 2");

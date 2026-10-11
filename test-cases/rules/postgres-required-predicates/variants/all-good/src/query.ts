import { query, sql } from "@example/db";
query(flag ? "SELECT id FROM accounts WHERE active IS TRUE" : "SELECT id FROM accounts WHERE active IS TRUE ORDER BY id");

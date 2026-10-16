import { query, sql } from "@example/db";
query(flag ? "SELECT 1" : "SELECT 2");
query(flag ? sql`SELECT ${value}` : sql`SELECT 3`);

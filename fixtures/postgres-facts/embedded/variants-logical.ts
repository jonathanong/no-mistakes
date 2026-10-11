import { query, sql } from "@example/db";
query((flag ? "SELECT 1" : "") || "SELECT 2");
query((flag ? "SELECT 3" : "SELECT 4") ?? "SELECT 5");
query(null ?? "SELECT 6");
query(sql`SELECT 1 ${flag && sql`LIMIT 1`}`);

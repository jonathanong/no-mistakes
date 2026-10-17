import { query, sql } from "@example/db";
query(sql`SELECT id FROM users ${flag ? sql`WHERE active` : sql`WHERE deleted`} LIMIT 1`);
query(sql`SELECT id FROM users ${sql.raw("WHERE active")} LIMIT 1`);
query(sql`SELECT id FROM users ${sql.join([sql`WHERE active`, sql`AND enabled`], " ")} LIMIT 1`);
query(sql`SELECT id FROM users ${flag ? sql`WHERE active` : "a bind value"} LIMIT 1`);

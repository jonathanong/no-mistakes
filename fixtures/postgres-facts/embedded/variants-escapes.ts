import { query, sql } from "@example/db";
const tail = flag
  ? sql`WHERE name = 'caf\u00e9'\n\
 OFFSET 1`
  : sql`WHERE active\n\
 LIMIT 1`;
query(sql`SELECT id FROM users ${tail}`);

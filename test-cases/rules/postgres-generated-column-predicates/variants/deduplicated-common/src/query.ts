import { query, sql } from "@example/db";
query(sql`SELECT id FROM orders WHERE created_at > $1 ${flag ? sql`/* first */` : sql`/* second */`}`);

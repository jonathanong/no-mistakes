import { query, sql } from "@example/db";
query(sql`UPDATE items SET created_at = now() WHERE id = $1 ${flag ? sql`/* first */` : sql`/* second */`}`);

import { query, sql } from "@example/db";
query(sql`SELECT 1 ${flag ? sql`/* first */` : sql`/* second */`}`);

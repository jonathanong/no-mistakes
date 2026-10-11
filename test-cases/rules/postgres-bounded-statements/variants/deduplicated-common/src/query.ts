import { query, sql } from "@example/db";
query(sql`SELECT id FROM accounts ${flag ? sql`/* first */` : sql`/* second */`}`);

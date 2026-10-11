import { query, sql } from "@example/db";
query(sql`SELECT *, * FROM accounts ${flag ? sql`/* first */` : sql`/* second */`}`);

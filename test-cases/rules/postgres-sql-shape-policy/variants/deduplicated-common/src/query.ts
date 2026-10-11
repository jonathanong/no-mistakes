import { query, sql } from "@example/db";
query(sql`SELECT pg_sleep(1) ${flag ? sql`/* first */` : sql`/* second */`}`);

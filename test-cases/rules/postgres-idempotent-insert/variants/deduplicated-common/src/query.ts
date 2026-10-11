import { query, sql } from "@example/db";
query(sql`INSERT INTO items (id) VALUES (1) ${flag ? sql`/* first */` : sql`/* second */`}`);

import { query, sql } from "@example/db";
query(sql`SELECT id FROM accounts WHERE id IN ($1, $2) FOR UPDATE ${flag ? sql`/* first */` : sql`/* second */`}`);

import { query, sql } from "@example/db";
query(sql`SELECT id FROM accounts ${(undefined && sql`LIMIT 1`) ?? sql`OFFSET 1`}`);

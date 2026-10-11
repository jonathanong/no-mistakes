import { query, sql } from "@example/db";
query(sql`SELECT pg_sleep(1), pg_sleep(1)`);

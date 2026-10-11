import { query, sql } from "@example/db";
query(sql`SELECT 1 ${sql`ignored` && sql`LIMIT 1`}`);
query(sql`SELECT 2 ${sql`` && sql`LIMIT 2`}`); // Even an empty builder is a truthy object.
query(sql`SELECT 3 ${sql.raw("") && sql`LIMIT 3`}`);
query(sql`SELECT 4 ${(flag ? sql`` : sql`ignored`) && sql`LIMIT 4`}`);

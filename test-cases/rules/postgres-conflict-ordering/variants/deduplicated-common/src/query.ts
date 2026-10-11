import { query, sql } from "@example/db";
query(sql`INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ON CONFLICT (id) DO NOTHING ${flag ? sql`/* first */` : sql`/* second */`}`);

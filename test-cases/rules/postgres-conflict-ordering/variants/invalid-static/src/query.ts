import { query, sql } from "@example/db";
query(sql`${flag ? sql`-- parse failure stays a statement diagnostic
  INSERT INTO items (id) SELECT ON CONFLICT (id) DO
` : sql`INSERT INTO items (id) VALUES ($1) ON CONFLICT (id) DO NOTHING`}`);

import { query, sql } from "@example/db";
query(sql`${flag ? sql`
  INSERT INTO items (id, note) VALUES ($1, 'n') ON CONFLICT (id) DO UPDATE SET note = items.note || EXCLUDED.note -- findings: insert
` : sql`INSERT INTO items (id, note) VALUES ($1, 'n') ON CONFLICT (id) DO UPDATE SET note = EXCLUDED.note`}`);
query(sql`${flag ? sql`
  INSERT INTO items (id) SELECT id FROM source_items -- findings: insert
` : sql`INSERT INTO items (id) SELECT id FROM source_items WHERE NOT EXISTS (SELECT 1 FROM items WHERE items.id = source_items.id)`}`);

import { query, sql } from "@example/db";
query(sql`UPDATE items SET ${flag ? sql`
  created_at = now() -- findings: created_at
` : sql`note = $1`} WHERE id = $2`);
query(sql`${flag ? sql`
  INSERT INTO items VALUES ($1, now(), 'n') -- findings: created_at
` : sql`INSERT INTO items (id, note) VALUES ($1, 'n')`}`);
query(sql`${flag ? sql`
  INSERT INTO items SELECT * FROM source_items -- findings: created_at
` : sql`INSERT INTO items (id, note) SELECT id, note FROM source_items`}`);
query(sql`MERGE INTO items t USING source_items s ON t.id = s.id WHEN MATCHED THEN UPDATE SET ${flag ? sql`
  created_at = now() -- findings: created_at
` : sql`note = s.note`}`);
query(sql`${flag ? sql`
  MERGE INTO items t USING source_items s ON t.id = s.id WHEN NOT MATCHED THEN INSERT VALUES (s.id, now(), s.note) -- findings: created_at
` : sql`MERGE INTO items t USING source_items s ON t.id = s.id WHEN NOT MATCHED THEN INSERT (id, note) VALUES (s.id, s.note)`}`);

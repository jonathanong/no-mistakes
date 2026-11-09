import { query, sql } from "db";

export async function recoveredStatements() {
  await query(sql`
    -- complete statement before the unsupported chunk
    SELECT DISTINCT id FROM posts;
    -- only an incomplete prefix is recovered from this chunk
    INSERT INTO current_posts SELECT DISTINCT id FROM posts WHERE id::text COLLATE "C" = $1;
    -- complete statement after the unsupported chunk
    SELECT DISTINCT id FROM posts;
  `);
}

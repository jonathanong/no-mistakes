import { query, sql } from '@example/db';

export function run(noteId: number) {
  return query(sql`SELECT COALESCE((SELECT m.role IN ('owner','moderator') FROM members m), false)
    AND EXISTS (SELECT 1 FROM notes n WHERE n.id = ${noteId})`);
}

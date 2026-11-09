import { query } from '@example/db';

export function rawNumbered() {
  return query(`SELECT COALESCE((SELECT m.role IN ('owner','moderator') FROM members m
    WHERE m.group_id = $1 AND m.user_id >= $2 AND m.user_id <= $2 AND m.removed_at IS NULL), false)
    AND EXISTS (SELECT 1 FROM notes n WHERE n.id = $3)`);
}

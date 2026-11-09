import { query, sql } from '@example/db';

export function run(groupId: number, userId: number, noteId: number) {
  const mutate = () => ++userId;
  return query(sql`SELECT COALESCE((SELECT m.role IN ('owner','moderator') FROM members m
    WHERE m.group_id = ${groupId} AND m.user_id >= ${userId} AND ${mutate()} IS NOT NULL AND m.user_id <= ${userId} AND m.removed_at IS NULL), false)
    AND EXISTS (SELECT 1 FROM notes n WHERE n.id = ${noteId})`);
}

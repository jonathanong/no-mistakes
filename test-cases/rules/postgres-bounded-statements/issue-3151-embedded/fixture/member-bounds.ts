import { query, sql } from '@example/db';

export function run(groupId: number, input: { userId: number }, noteId: number) {
  return query(sql`SELECT COALESCE((SELECT m.role IN ('owner','moderator') FROM members m
    WHERE m.group_id = ${groupId} AND m.user_id >= ${input.userId} AND m.user_id <= ${input.userId} AND m.removed_at IS NULL), false)
    AND EXISTS (SELECT 1 FROM notes n WHERE n.id = ${noteId})`);
}

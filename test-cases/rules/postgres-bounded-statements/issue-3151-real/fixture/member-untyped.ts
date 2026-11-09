import SQL from 'sql-template-strings';
import { query } from '@example/db';

export function isModerator(user, groupId) {
  return query(SQL`SELECT COALESCE((SELECT m.role IN ('owner','moderator') FROM group_members m
     WHERE m.group_id = ${groupId} AND m.user_id >= ${user.id} AND m.user_id <= ${user.id}
       AND m.removed_at IS NULL), false)
   AND EXISTS (SELECT 1 FROM groups g WHERE g.id = ${groupId} AND g.deleted_at IS NULL) AS is_moderator`);
}

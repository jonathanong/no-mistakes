import { beginTransaction as begin } from '@example/db';
import sql from 'sql-template-strings';
export async function renamed(ownerId: string, userIds: string[]) {
  await using query = await begin({ timeout: 20 });
  const statement = sql`INSERT INTO group_members (group_id, user_id) VALUES (1, ${ownerId})`;
  for (const userId of userIds) statement.append(sql`, (1, ${userId})`);
  return query(statement);
}

import { read, beginTransaction } from '@example/db';
import sql from 'sql-template-strings';
export async function reassigned(ownerId: string, userIds: string[]) {
  beginTransaction = unknownFactory;
  await using query = await beginTransaction();
  const statement = sql`INSERT INTO group_members (group_id, user_id) VALUES (1, ${ownerId})`;
  for (const userId of userIds) statement.append(sql`, (1, ${userId})`);
  return query(statement);
}

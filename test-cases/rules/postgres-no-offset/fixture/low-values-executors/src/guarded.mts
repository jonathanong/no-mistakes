import { read, beginTransaction } from '@example/db';
import sql from 'sql-template-strings';
export async function guarded(ownerId: string, userIds: string[]) {
  if (userIds.length < 1) throw new Error('Provide at least one member');
  await using query = await beginTransaction();
  const { rows } = await query(sql`INSERT INTO groups (owner_id) VALUES (${ownerId}) RETURNING id`);
  const group = rows[0];
  const statement = sql`INSERT INTO group_members (group_id, user_id) VALUES (${group.id}, ${ownerId})`;
  for (const userId of userIds) statement.append(sql`, (${group.id}, ${userId})`);
  await query(statement);
  await query.commit();
  return group;
}
export async function thrownSql(ownerId: string, userIds: string[]) {
  if (userIds.length < 1) throw new Error(await read(sql`SELECT 1 OFFSET 9`));
  await using query = await beginTransaction();
  const statement = sql`INSERT INTO group_members (group_id, user_id) VALUES (1, ${ownerId})`;
  for (const userId of userIds) statement.append(sql`, (1, ${userId})`);
  return query(statement);
}
export async function thrownExecutor(ownerId: string, userIds: string[]) {
  if (userIds.length < 1) throw new Error(read);
  await using query = await beginTransaction();
  const statement = sql`INSERT INTO group_members (group_id, user_id) VALUES (1, ${ownerId})`;
  for (const userId of userIds) statement.append(sql`, (1, ${userId})`);
  return query(statement);
}

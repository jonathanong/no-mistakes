import { read } from '@example/db';
import sql from 'sql-template-strings';
export function fixed(groupId: string, userId: string) {
  return read(sql`INSERT INTO group_members (group_id, user_id) VALUES (${groupId}, ${userId})`);
}
export function repeated(groupId: string, ownerId: string, userIds: string[]) {
  const statement = sql`INSERT INTO group_members (group_id, user_id) VALUES (${groupId}, ${ownerId})`;
  for (const userId of userIds) {
    statement.append(sql`, (${groupId}, ${userId})`);
  }
  return read(statement);
}
export function structural(table: string, userIds: string[]) {
  const statement = sql`INSERT INTO group_members (group_id, user_id) VALUES (1, 2)`;
  for (const userId of userIds) {
    statement.append(`; DELETE FROM ${table}`);
  }
  return read(statement);
}
export async function returned(ownerId: string, userIds: string[]) {
  const { rows } = await read(sql`INSERT INTO groups (owner_id) VALUES (${ownerId}) RETURNING id`);
  const group = rows[0];
  const statement = sql`INSERT INTO group_members (group_id, user_id) VALUES (${group.id}, ${ownerId})`;
  for (const userId of userIds) {
    statement.append(sql`, (${group.id}, ${userId})`);
  }
  return read(statement);
}
import { beginTransaction } from '@example/db';
export async function transaction(ownerId: string, userIds: string[]) {
  await using query = await beginTransaction();
  const { rows } = await query(sql`INSERT INTO groups (owner_id) VALUES (${ownerId}) RETURNING id`);
  const group = rows[0];
  const statement = sql`INSERT INTO group_members (group_id, user_id) VALUES (${group.id}, ${ownerId})`;
  for (const userId of userIds) {
    statement.append(sql`, (${group.id}, ${userId})`);
  }
  await query(statement);
  return group;
}
export async function opaqueArgument(ownerId: string, userIds: string[]) {
  await using query = await beginTransaction(unknownHelper());
  const statement = sql`INSERT INTO group_members (group_id, user_id) VALUES (1, ${ownerId})`;
  for (const userId of userIds) statement.append(sql`, (1, ${userId})`);
  return query(statement);
}
export async function escapedExecutor(ownerId: string, userIds: string[]) {
  await using query = await beginTransaction(read);
  const statement = sql`INSERT INTO group_members (group_id, user_id) VALUES (1, ${ownerId})`;
  for (const userId of userIds) statement.append(sql`, (1, ${userId})`);
  return query(statement);
}
export async function sqlArgument(ownerId: string, userIds: string[]) {
  await using query = await beginTransaction(sql`SELECT 1 OFFSET 7`);
  const statement = sql`INSERT INTO group_members (group_id, user_id) VALUES (1, ${ownerId})`;
  for (const userId of userIds) statement.append(sql`, (1, ${userId})`);
  return query(statement);
}
export async function argumentEffect(ownerId: string, userIds: string[]) {
  await using query = await beginTransaction(await read(sql`SELECT 1 OFFSET 8`));
  const statement = sql`INSERT INTO group_members (group_id, user_id) VALUES (1, ${ownerId})`;
  for (const userId of userIds) statement.append(sql`, (1, ${userId})`);
  return query(statement);
}

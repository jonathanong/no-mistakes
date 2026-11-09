import { query } from '@example/db';

export function legacySql() {
  const user = { table: 'group_members' };
  return query(`SELECT role FROM ${user.table}`);
}

export function escapedOwnData(userId: number) {
  const user = { id: userId };
  const alias = user;
  return alias.id;
}

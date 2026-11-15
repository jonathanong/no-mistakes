import { query } from '@example/db';
import sql from 'sql-template-strings';

const relationTables = Object.freeze({
  vote: 'users',
  tag: 'posts',
} as const);

const relationConfigs = Object.freeze({
  vote: { table: 'users', column: 'user_id' },
  tag: { table: 'posts', column: 'post_id' },
} as const);

export function getRelationTable(name: string) {
  if (!Object.hasOwn(relationTables, name)) throw new Error(`unknown ${name}`);
  return relationTables[name as keyof typeof relationTables];
}

export function getRelationConfig(name: string) {
  if (!Object.hasOwn(relationConfigs, name)) throw new Error(`unknown ${name}`);
  return relationConfigs[name as keyof typeof relationConfigs];
}

export function run(kind: string) {
  query(`/* both */ DELETE FROM ${getRelationTable(kind)} WHERE id = $1`);
  const config = getRelationConfig(kind);
  query(`/* corr */ DELETE FROM ${config.table} WHERE ${config.column} = $1`);
  query(`/* one */ DELETE FROM ${getRelationTable('vote')} WHERE id = $1`);
  query(`/* miss */ DELETE FROM ${getRelationTable('missing')} WHERE id = $1`);
  const statement = sql`/* append */ DELETE FROM `;
  statement.append(getRelationTable(kind));
  statement.append(' WHERE id = $1');
  query(statement);
}

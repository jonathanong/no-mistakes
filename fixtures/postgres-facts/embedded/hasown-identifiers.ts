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
  const one = getRelationConfig('vote');
  query(`/* one-config */ DELETE FROM ${one.table} WHERE ${one.column} = $1`);
  const statement = sql`/* append */ DELETE FROM `;
  statement.append(getRelationTable(kind));
  statement.append(' WHERE id = $1');
  query(statement);
}

export function unionConfig(kind: 'vote' | 'tag') {
  const config = getRelationConfig(kind);
  query(`/* union-config */ DELETE FROM ${config.table} WHERE ${config.column} = $1`);
}

export function partialConfig(kind: 'vote' | 'missing') {
  const config = getRelationConfig(kind);
  query(`/* partial-config */ DELETE FROM ${config.table} WHERE ${config.column} = $1`);
}

export function objectKey() {
  const config = getRelationConfig({ vote: 'vote' });
  query(`/* object-key */ DELETE FROM ${config.table} WHERE id = $1`);
}

export function optionalField(kind: string) {
  const config = getRelationConfig(kind);
  query(`/* optional */ DELETE FROM ${config?.table} WHERE id = $1`);
}

const frozenConfigs = Object.freeze({
  a: Object.freeze({ table: 'retained_a', column: 'a_id' }),
  b: Object.freeze({ table: 'retained_b', column: 'b_id' }),
});

export function getFrozenConfig(name: string) {
  if (!Object.hasOwn(frozenConfigs, name)) throw new Error(`unknown ${name}`);
  return frozenConfigs[name as keyof typeof frozenConfigs];
}

export function insertFrozen(name: string) {
  const config = getFrozenConfig(name);
  query(`/* frozen-column */ INSERT INTO impacts (request_id, ${config.column}) VALUES ($1, $2)`);
  query(`/* frozen-table */ INSERT INTO ${config.table} (id) VALUES ($1)`);
}

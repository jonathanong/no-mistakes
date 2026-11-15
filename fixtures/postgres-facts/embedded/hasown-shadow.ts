import { query } from '@example/db';

function Object() {
  return null;
}

const tables = Object.freeze({
  vote: 'users',
  tag: 'posts',
} as const);

export function getTable(name: string) {
  if (!Object.hasOwn(tables, name)) throw new Error('no');
  return tables[name as keyof typeof tables];
}

export function run(kind: string) {
  query(`/* shadow */ DELETE FROM ${getTable(kind)} WHERE id = $1`);
}

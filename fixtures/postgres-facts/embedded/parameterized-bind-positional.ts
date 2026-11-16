import SQL from 'sql-template-strings';
import { query } from '@example/db';

export function run(id: string) {
  query(SQL`SELECT $${10}, ${id}, $${20}, ${'active'}`);
}

import { it } from 'vitest';
import { query } from '@example/db';
import { each as untrustedEach } from '@other/test';

declare const value: string;
declare function consume(value: unknown): void;

it.each(['first', 'second', 'third'] as const)('finite %s', async field => {
  const updates = {
    first: { sql: 'first_id = $2', value },
    second: { sql: 'second_id = $2', value },
    third: { sql: 'third_id = $2', value },
  };
  const update = updates[field];
  await query(`/* finite */ UPDATE records SET ${update.sql} WHERE id = $1`, ['id', update.value]);
});

it.each(['read', 'drop'])('finite schema %s', async field => {
  const statements = {
    read: { sql: 'SELECT 1' },
    drop: { sql: 'DROP TABLE records' },
  };
  const selected = statements[field];
  await query(`/* finite_schema */ ${selected.sql}`);
});

untrustedEach(['first', 'second'])('unknown %s', async field => {
  const updates = { first: { sql: 'first_id = $2' }, second: { sql: 'second_id = $2' } };
  const update = updates[field];
  await query(`/* untrusted */ UPDATE records SET ${update.sql} WHERE id = $1`);
});

it.each(['first', 'second'] as const)('reassigned %s', async field => {
  const updates = { first: { sql: 'first_id = $2' }, second: { sql: 'second_id = $2' } };
  field = '__proto__' as any;
  const update = updates[field];
  await query(`/* reassigned */ UPDATE records SET ${update.sql} WHERE id = $1`);
});

it.each(['first', 'second'] as const)('escaped %s', async field => {
  const updates = { first: { sql: 'first_id = $2' }, second: { sql: 'second_id = $2' } };
  consume(updates);
  const update = updates[field];
  await query(`/* escaped */ UPDATE records SET ${update.sql} WHERE id = $1`);
});

it.each(['first', 'missing'] as const)('partial %s', async field => {
  const updates = { first: { sql: 'first_id = $2' } };
  const update = updates[field];
  await query(`/* partial */ UPDATE records SET ${update.sql} WHERE id = $1`);
});

it.each(['first'])('deleted %s', async field => {
  const updates = { first: { sql: 'first_id = $2' } };
  const update = updates[field];
  delete update.sql;
  await query(`/* deleted */ UPDATE records SET ${update.sql} WHERE id = $1`);
});

it.each(['first'])('prototype setter %s', async field => {
  // This syntax changes the record prototype; it is not an own data key.
  const updates = { first: { __proto__: { sql: 'other_id = $2' }, sql: 'first_id = $2' } };
  const update = updates[field];
  await query(`/* prototype */ UPDATE records SET ${update.sql} WHERE id = $1`);
});

it.each(['first'])('shorthand sql %s', async field => {
  const sql = 'first_id = $2';
  // The selected SQL field itself still needs an own literal value.
  const updates = { first: { sql } };
  const update = updates[field];
  await query(`/* shorthand */ UPDATE records SET ${update.sql} WHERE id = $1`);
});

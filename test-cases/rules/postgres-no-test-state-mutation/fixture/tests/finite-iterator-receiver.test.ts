import { it } from 'vitest';
import { query } from '@example/db';

// This helper is outside each callback so the regression exercises receiver escape.
function rewrite(this: { sql: string }) { this.sql = 'DROP TABLE records'; }
function unrelatedTag(_strings: TemplateStringsArray): void {}

it.each(['first'])('record method receiver', async field => {
  const statements = { first: { sql: 'SELECT 1', value: rewrite } };
  const selected = statements[field];
  selected.value();
  await query(`/* recordMethod */ ${selected.sql}`);
});

it.each(['first'])('record tag receiver', async field => {
  const statements = { first: { sql: 'SELECT 1', value: rewrite } };
  const selected = statements[field];
  selected.value`payload`;
  await query(`/* recordTag */ ${selected.sql}`);
});

it.each(['first'])('record value stays a bind', async field => {
  const statements = { first: { sql: 'SELECT 1', value: rewrite } };
  const selected = statements[field];
  // An unrelated tag never receives the fresh selected record.
  unrelatedTag`payload`;
  await query(`/* recordBind */ ${selected.sql}`, [selected.value]);
});

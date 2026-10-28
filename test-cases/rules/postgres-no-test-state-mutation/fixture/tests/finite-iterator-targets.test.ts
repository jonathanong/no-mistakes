import { it } from 'vitest';
import { query } from '@example/db';

it.each(['first'])('for-of writes the selected SQL', async field => {
  const statements = { first: { sql: 'SELECT 1' } };
  const selected = statements[field];
  for (selected.sql of ['DROP TABLE records']) {}
  await query(`/* assignmentOf */ ${selected.sql}`);
});
it.each(['first'])('for-in writes the selected SQL', async field => {
  const statements = { first: { sql: 'SELECT 1' } };
  const selected = statements[field];
  for (selected.sql in { 'DROP TABLE records': 1 }) {}
  await query(`/* assignmentIn */ ${selected.sql}`);
});
it.each(['first'])('fresh iteration binding is independent', async field => {
  const statements = { first: { sql: 'SELECT 1' } };
  const selected = statements[field];
  for (const ignored of ['one']) {}
  await query(`/* independentIteration */ ${selected.sql}`);
});

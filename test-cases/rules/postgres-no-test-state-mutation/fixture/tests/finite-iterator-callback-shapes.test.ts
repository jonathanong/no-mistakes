import { it } from 'vitest';
import { query } from '@example/db';

// A non-arrow callback may observe a different runtime scope.
it.each(['first'])('function callback', function (field) {
  const map = { first: { sql: 'safe_one' } };
  const selected = map[field];
  return query(`/* functionCallback */ SELECT 1 FROM ${selected.sql}`);
});

// Extra or defaulted parameters can change the callback's runtime key.
it.each(['first'])('rest callback', (field, ...rest) => {
  const map = { first: { sql: 'safe_one' } };
  const selected = map[field];
  return query(`/* restCallback */ SELECT 1 FROM ${selected.sql}`);
});
it.each(['first'])('default callback', (field = 'first') => {
  const map = { first: { sql: 'safe_one' } };
  const selected = map[field];
  return query(`/* defaultCallback */ SELECT 1 FROM ${selected.sql}`);
});
it.each(['first'])('destructured callback', ({ field }) => {
  const map = { first: { sql: 'safe_one' } };
  const selected = map[field];
  return query(`/* destructuredCallback */ SELECT 1 FROM ${selected.sql}`);
});

// An expression body has no local declaration and selection proof.
it.each(['first'])('expression callback', field =>
  query(`/* expressionCallback */ SELECT 1 FROM ${field}`));

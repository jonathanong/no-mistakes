import { it } from 'vitest';
import { query } from '@example/db';
declare const seed: any;
declare const keys: string[];
declare function consume(value: unknown): void;

// Runtime SQL remains unknown: runtimeKeys.
it.each(keys)('runtimeKeys %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = map[field];
  await query(`/* runtimeKeys */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: emptyKeys.
it.each([])('emptyKeys %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = map[field];
  await query(`/* emptyKeys */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: numericKey.
it.each([1])('numericKey %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = map[field];
  await query(`/* numericKey */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: duplicateKeys.
it.each(['first', 'first'])('duplicateKeys %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = map[field];
  await query(`/* duplicateKeys */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: spreadMap.
it.each(['first'])('spreadMap %s', async field => {
  const map = { ...seed, first: { sql: 'safe_one' } };
  const selected = map[field];
  await query(`/* spreadMap */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: computedMap.
it.each(['first'])('computedMap %s', async field => {
  const map = { [field]: { sql: 'safe_one' } };
  const selected = map[field];
  await query(`/* computedMap */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: prototypeMap.
it.each(['first'])('prototypeMap %s', async field => {
  const map = { __proto__: seed, first: { sql: 'safe_one' } };
  const selected = map[field];
  await query(`/* prototypeMap */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: primitiveRecord.
it.each(['first'])('primitiveRecord %s', async field => {
  const map = { first: 'safe_one' };
  const selected = map[field];
  await query(`/* primitiveRecord */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: spreadRecord.
it.each(['first'])('spreadRecord %s', async field => {
  const map = { first: { ...seed, sql: 'safe_one' } };
  const selected = map[field];
  await query(`/* spreadRecord */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: computedField.
it.each(['first'])('computedField %s', async field => {
  const map = { first: { ['sql']: 'safe_one' } };
  const selected = map[field];
  await query(`/* computedField */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: prototypeField.
it.each(['first'])('prototypeField %s', async field => {
  const map = { first: { __proto__: seed, sql: 'safe_one' } };
  const selected = map[field];
  await query(`/* prototypeField */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: numericSql.
it.each(['first'])('numericSql %s', async field => {
  const map = { first: { sql: 1 } };
  const selected = map[field];
  await query(`/* numericSql */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: duplicateMap.
it.each(['first'])('duplicateMap %s', async field => {
  const map = { first: { sql: 'safe_one' }, first: { sql: 'safe_two' } };
  const selected = map[field];
  await query(`/* duplicateMap */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: mutableMap.
it.each(['first'])('mutableMap %s', async field => {
  let map = { first: { sql: 'safe_one' } };
  const selected = map[field];
  await query(`/* mutableMap */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: destructuredMap.
it.each(['first'])('destructuredMap %s', async field => {
  const { map } = seed;
  const selected = map[field];
  await query(`/* destructuredMap */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: optionalSelection.
it.each(['first'])('optionalSelection %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = map?.[field];
  await query(`/* optionalSelection */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: wrongSelection.
it.each(['first'])('wrongSelection %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = seed[field];
  await query(`/* wrongSelection */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: changedSelector.
it.each(['first'])('changedSelector %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = map[field];
  field++;
  await query(`/* changedSelector */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: deletedField.
it.each(['first'])('deletedField %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = map[field];
  delete selected.sql;
  await query(`/* deletedField */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: nestedFunction.
it.each(['first'])('nestedFunction %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = map[field];
  const captured = function () { return selected.sql }; consume(captured);
  await query(`/* nestedFunction */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: nestedArrow.
it.each(['first'])('nestedArrow %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = map[field];
  const captured = () => selected.sql; consume(captured);
  await query(`/* nestedArrow */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: nestedClass.
it.each(['first'])('nestedClass %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = map[field];
  const Captured = class { value() { return selected.sql } }; consume(Captured);
  await query(`/* nestedClass */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

// Runtime SQL remains unknown: shadowedAlias.
it.each(['first'])('shadowedAlias %s', async field => {
  const map = { first: { sql: 'safe_one', value: 1 }, second: { sql: 'safe_two', value: 2 } };
  const selected = map[field];
  { const selected = seed; consume(selected); }
  await query(`/* shadowedAlias */ SELECT 1 FROM ${selected.sql}`, [selected.value]);
});

import { read, sql } from '@example/db';
function atLimit(a: boolean, b: boolean, c: boolean, d: boolean) {
  return `/* variants/at-limit */ SELECT ${a ? '1' : '2'} AS a, ${b ? '1' : '2'} AS b, ${c ? '1' : '2'} AS c, ${d ? '1' : '2'} AS d`;
}
function pastLimit(a: boolean, b: boolean, c: boolean, d: boolean, e: boolean) {
  return `${atLimit(a, b, c, d)}, ${e ? '1' : '2'} AS e`;
}
export function limits(a: boolean, b: boolean, c: boolean, d: boolean, e: boolean) {
  read(atLimit(a, b, c, d));
  read(pastLimit(a, b, c, d, e));
}
function mixed(flag: boolean) { return flag ? 'SELECT 1' : runtime.sql; }
export function opaqueAlternative(flag: boolean) { read(mixed(flag)); }

function conditionallyAppend(query: unknown, flag: boolean) {
  if (flag) query.append(sql` removed AS (DELETE FROM records RETURNING id) `);
  else query.append(sql` removed AS (SELECT id FROM records) `);
}
export function outsideContinuation(flag: boolean) {
  const query = sql`/* append/mixed */ WITH `;
  conditionallyAppend(query, flag);
  query.append(sql` SELECT id FROM removed`);
  read(query);
}

function conditionalShape(flag: boolean) {
  const parts = [];
  flag ? parts.push('1') : parts.push('1', '2');
  return `/* collections/shape */ SELECT ${parts.join(', ')}`;
}
export function shape(flag: boolean) { read(conditionalShape(flag)); }
export function spreadShape(input: unknown[]) {
  read(`/* collections/spread-shape */ WITH ${[...input].map(() => 'part AS (SELECT 1)').join(', ')} SELECT 1`);
}
export function getterEffect() {
  const query = sql`/* effects/getter */ SELECT 1`;
  const obj = { get text() { query.append(sql`; DELETE FROM records`); return '1'; } };
  const text = obj.text;
  read(query);
}
export function opaqueEffect(callback: unknown) {
  const query = sql`/* effects/callback */ SELECT 1`;
  ['1'].forEach(callback);
  read(query);
}
export function localBinding(flag: boolean) {
  if (flag) { const text = '/* branches/local */ SELECT 1'; read(text); }
}
export function sparseArray() { read(`/* collections/sparse */ SELECT ${[, '1'].join(', ')}`); }
export function builderJoin() { read(`/* collections/builder-join */ SELECT ${[sql`1`].join(', ')}`); }

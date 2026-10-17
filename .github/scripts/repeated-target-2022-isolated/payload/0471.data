import { read, sql } from '@example/db';
// A lexical SELECT from join must not certify a monkey-patched intrinsic.
Array.prototype.join = function () { return 'DELETE FROM records'; };
export function patchedJoin() {
  read(['/* intrinsics/join */ SELECT 1'].join(''));
}
export function patchedMap() {
  read(['/* intrinsics/map */ SELECT 2'].map(value => value).join(''));
}
export function patchedPush() {
  const parts = ['/* intrinsics/push */ SELECT 3'];
  parts.push('');
  read(parts.join(''));
}
export function patchedForEach() {
  const query = sql`/* intrinsics/for-each */ SELECT 4`;
  [''].forEach(() => query.append('; DELETE FROM records'));
  read(query);
}

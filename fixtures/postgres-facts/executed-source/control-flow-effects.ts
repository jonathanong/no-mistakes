import { read, sql } from '@example/db';
export function separator(separator: unknown) { read(`/* effects/separator */ SELECT ${['1'].join(separator)}`); }
export function asynchronousMap() { read(`/* effects/async-map */ SELECT ${['1'].map(async value => value).join(', ')}`); }
export function unsupportedMap() { read(`/* effects/unsupported-map */ SELECT ${['1'].map((...values) => values[0]).join(', ')}`); }
export function capturedCallbackEffect() {
  const query = sql`/* effects/captured */ SELECT 1`;
  ['1'].forEach(() => { arbitraryMutation(); });
  read(query);
}
export function opaqueCondition() {
  const query = sql`/* effects/condition */ SELECT 1`;
  if (opaqueTest()) query.append(sql` WHERE true`);
  read(query);
}
export function sparseMutation() {
  const parts = ['1'];
  parts[9] = '2';
  read(`/* effects/sparse-mutation */ SELECT ${parts.join(', ')}`);
}
export function spreadArguments(values: unknown[]) { read(`/* effects/spread-arguments */ SELECT ${['1'].map(...values).join(', ')}`); }
export function liveMapElements() {
  const parts = ['first AS (SELECT 1)', 'second AS (SELECT 2)'];
  const ctes = parts.map((value, index) => {
    if (index === 0) parts[1] = 'second AS (DELETE FROM records RETURNING id)';
    return value;
  });
  read(`/* effects/live-elements */ WITH ${ctes.join(', ')} SELECT * FROM first`);
}
export function conditions() {
  const parts = [1, 2].map((value, index) => {
    if (index != 0) return '2';
    if (index <= 0) return '1';
    return '3';
  });
  read(`/* effects/comparisons */ SELECT ${parts.join()}`);
}
export function unknownReturningCondition() {
  const query = sql`/* effects/returned-condition */ SELECT 1`;
  function helper() { return opaqueTest() ? 'one' : 'two'; }
  const result = helper();
  read(query);
}
export function mappedBranches(flag: boolean) {
  const parts = ['first', 'second'].map(name => flag ? `${name} AS (SELECT 1)` : `${name} AS (SELECT 2)`);
  read(`/* effects/mapped-branches */ WITH ${parts.join(', ')} SELECT * FROM first`);
}
export function unsupportedArithmetic() { read(`/* effects/arithmetic */ SELECT ${3 % 2}`); }
export function deletedElement() {
  const parts = ['1', '2'];
  delete parts[0];
  read(`/* effects/deletion */ SELECT ${parts.join(', ')}`);
}
function unrelatedDeletion(value: unknown) {
  delete arguments[0];
  read(`/* effects/unrelated-deletion */ SELECT ${['1'].join(', ')}`);
}
export function unchangedArray() { unrelatedDeletion('unused'); }
export function overflow() { read(`/* effects/overflow */ SELECT ${9007199254740990 + 2}`); }
function falseBranch() {
  if (false) return '/* effects/false-branch */ WITH removed AS (DELETE FROM records RETURNING id) SELECT id FROM removed';
  return '/* effects/false-branch */ SELECT 1';
}
export function knownFalseBranch() { read(falseBranch()); }

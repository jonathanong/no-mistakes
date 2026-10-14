import { read, sql } from '@example/db';
function runArray() {
  const query = sql`/* returns/array */ SELECT 3`;
  query.append(['; DELETE FROM records'].join(''));
  read(query);
}
function runRecord() {
  const query = sql`/* returns/record */ SELECT 4`;
  query.append(['; DELETE FROM records'].join(''));
  read(query);
}
function runCycle() {
  const query = sql`/* returns/cycle */ SELECT 5`;
  query.append(['; DELETE FROM records'].join(''));
  read(query);
}
function cycleLeft() { return cycleRight; }
function cycleRight() { return cycleLeft; }
export function wrapped(flag: boolean) {
  if (flag) return [runArray][0];
  else return ({ run: runRecord }).run;
}
// Mutually returned functions have valid hoisted bindings. Reachability must
// stop their cycle while still finding the other branch's private entrypoint.
export function cyclic(flag: boolean) {
  if (flag) return cycleLeft;
  else return runCycle;
}
const callbacks = [runArray, runRecord];
export function expressionWrappers(flag: boolean, index: number) {
  if (flag) return index ? runArray : runRecord;
  else return [runArray, runRecord][index];
}
export function collectionWrappers(flag: boolean) {
  if (flag) return [...callbacks];
  else return runCycle && runCycle;
}

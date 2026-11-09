import { owner } from './owner.mjs';
function makeObject() { return {}; }
function predicate() { return false; }
export function repeatedFind(rows: number[], opaque: (value: unknown) => boolean) {
  return rows.map(makeObject).find(predicate).find(opaque);
}
export function numberMethods(rows: number[]) {
  rows.forEach(n => n.toExponential());
  rows.forEach(n => n.toPrecision());
}
export function repeatedFindOwner(rows: number[], opaque: (value: unknown) => boolean) {
  return rows.map(makeObject).find(owner).find(opaque);
}

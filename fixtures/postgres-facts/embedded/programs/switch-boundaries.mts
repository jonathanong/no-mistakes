import sql from 'sql-template-strings';
function abortBoth(flag) { if (flag) throw Error('left'); else throw Error('right'); }
export function returningDefault() { return sql`SELECT 2`; }
export function deferredValue(kind) {
  switch (kind) { case 'a': return sql`SELECT 1`; default: return returningDefault(); }
}
export function unknownDefault(kind) {
  switch (kind) { case 'a': return sql`SELECT 1`; default: return missingDefault(); }
}
export function throwingBranches(kind) {
  switch (kind) { case 'a': return sql`SELECT 1`; default: return abortBoth(kind); }
}
export function memberSelector(value) {
  switch (value.kind) { case 'a': return sql`SELECT 1`; default: throw Error('unknown'); }
}
export function failedArm(kind) {
  switch (kind) { case 'a': return sql`SELECT ${missing()}`; default: throw Error('unknown'); }
}
export function inertPrefix(kind) {
  switch (kind) { case 'a': return sql`SELECT 1`; default: { const value = kind; throw Error(value); } }
}
export function mutablePrefix(kind) {
  switch (kind) { case 'a': return sql`SELECT 1`; default: { let value = kind; throw Error(value); } }
}
export function multiplePrefix(kind) {
  switch (kind) { case 'a': return sql`SELECT 1`; default: { const first = kind, second = kind; throw Error(first); } }
}
export function destructuredPrefix(kind) {
  switch (kind) { case 'a': return sql`SELECT 1`; default: { const {value} = kind; throw Error(value); } }
}
export function computedPrefix(kind) {
  switch (kind) { case 'a': return sql`SELECT 1`; default: { const value = kind.length; throw Error(value); } }
}
export function expressionPrefix(kind) {
  switch (kind) { case 'a': return sql`SELECT 1`; default: { opaque(kind); throw Error('unknown'); } }
}
export function valuelessDefault(kind) {
  switch (kind) { case 'a': return sql`SELECT 1`; default: return; }
}
export function valuelessArm(kind) {
  switch (kind) { case 'a': return; case 'b': return sql`SELECT 1`; default: throw Error('unknown'); }
}

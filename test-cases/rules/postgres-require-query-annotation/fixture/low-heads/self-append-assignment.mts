import { read } from '@bench/db'
import sql from 'sql-template-strings'
import custom from '@bench/custom'
declare const n: number, flag: boolean, other: any
declare function outside(value: unknown): unknown
export function branches() { let q = sql`/* named */ SELECT ${n}`; if (flag) q = q.append(sql` WHERE x = ${n}`); else q = q.append(sql` WHERE y = ${n}`); q = q.append(sql` ORDER BY x`).append(sql` LIMIT ${n}`); read(q); read(q); }
export function opaqueSuffix() { let q = sql`/* named */ SELECT ${n}`; q = q.append(other); q.append(sql` LIMIT ${n}`); read(q); }
export function missing() { let q = sql`SELECT ${n}`; q = q.append(sql` WHERE x = ${n}`); read(q); }
export function newHeap() { let q = sql`/* named */ SELECT ${n}`; q = sql`/* different */ SELECT ${n}`; read(q); }
export function otherBinding() { let q = sql`/* named */ SELECT ${n}`; q = other.append(sql` suffix`); read(q); }
export function unknown() { let q = sql`/* named */ SELECT ${n}`; q = outside(q.append(sql` suffix`)); read(q); }
export function alias() { let q = sql`/* named */ SELECT ${n}`; const alias = q; q = q.append(sql` suffix`); read(q); }
export function pass() { let q = sql`/* named */ SELECT ${n}`; outside(q); q = q.append(sql` suffix`); read(q); }
export function capture() { let q = sql`/* named */ SELECT ${n}`; const captured = () => q; q = q.append(sql` suffix`); read(q); }
export function textMutation() { let q = sql`/* named */ SELECT ${n}`; q.text = "SELECT 2"; q = q.append(sql` suffix`); read(q); }
export function valuesEscape() { let q = sql`/* named */ SELECT ${n}`; outside(q.values); q = q.append(sql` suffix`); read(q); }
export function operator() { let q = sql`/* named */ SELECT ${n}`; q += q.append(sql` suffix`); read(q); }
export function usedResult() { let q = sql`/* named */ SELECT ${n}`; outside(q = q.append(sql` suffix`)); read(q); }
export function optional() { let q = sql`/* named */ SELECT ${n}`; q = q?.append(sql` suffix`); read(q); }
export function spread() { let q = sql`/* named */ SELECT ${n}`; q = q.append(...other); read(q); }
export function methodAlias() { let q = sql`/* named */ SELECT ${n}`; const append = q.append; q = append(sql` suffix`); read(q); }
export function prepend() { let q = sql`/* named */ SELECT ${n}`; q = other.append(q); read(q); }
export function loop() { let q = sql`/* named */ SELECT ${n}`; while (flag) q = q.append(sql` suffix`); read(q); }
export function discardedLoop() { let q = sql`/* named */ SELECT ${n}`; while (flag) q.append(sql` suffix`); q = q.append(sql` suffix`); read(q); }
export function overBudget() { let q = sql`/* named */ SELECT ${n}`; q = q.append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`); read(q); }
export function selfBind() { let q = sql`/* named */ SELECT ${n}`; q = q.append(sql`${q}`); read(q); }
export function nestedWrite() { let q = sql`/* named */ SELECT ${n}`; const alter = () => { q = q.append(sql` suffix`); }; q = q.append(sql` suffix`); read(q); }
export function unknownMethod() { let q = sql`/* named */ SELECT ${n}`; q.unknown(); q = q.append(sql` suffix`); read(q); }
export function sameNameShadow() { let q = sql`/* named */ SELECT ${n}`; { let q = other; q = q.append(sql` suffix`); } q = other; read(q); }
export function wrongTag() { let q = custom`/* named */ SELECT ${n}`; q = q.append(sql` suffix`); read(q); }
// EOF proofs must survive every suffix; repairing a prefix cannot inherit a safe head.
export function beginRepair() { let q = sql`BEGIN${n}`; q = q.append("NING"); read(q); }
export function commentRepair() { let q = sql`/*${n}`; q = q.append(" named */ SELECT 1"); read(q); }
export let exportedQ = sql`/* named */ SELECT ${n}`; exportedQ = exportedQ.append(sql` suffix`); read(exportedQ);
export function capturedAppend() { let q = sql`/* named */ SELECT ${n}`; const f = () => { q.append(sql` suffix`); }; q = q.append(sql` suffix`); read(q); }
export function descriptorEscape() { let q = sql`/* named */ SELECT ${n}`; Object.defineProperty(q, 'append', { value: outside }); q = q.append(sql` suffix`); read(q); }
export function tdz() { read(q); let q = sql`/* named */ SELECT ${n}`; q = q.append(sql` suffix`); }
export function emptyRepair() { let q = sql``; q = q.append('/* named */ SELECT 1'); read(q); }
export function eightLinks() { let q = sql`/* named */ SELECT ${n}`; q = q.append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`); read(q); }

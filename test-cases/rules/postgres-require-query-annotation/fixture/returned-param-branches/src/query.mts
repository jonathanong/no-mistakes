import { read } from '@bench/db'
import sql from 'sql-template-strings'
import custom from '@bench/custom'
import { factory, unproven } from './factories.mjs'
declare const flag: boolean, extra: any, other: any
declare function inspect(value: unknown): any
export function directOne() {  const q = factory(sql`/* named */ SELECT `);  read(q.text, q.values) // expect: Literal
}
export function directTwo() {  const q = factory(sql`/* directTwo */ SELECT `, extra);  read(q.text, q.values) // expect: Literal
}
export function conditionalOne() {  const q = flag ? factory(sql`/* one */ SELECT `) : factory(sql`/* two */ SELECT `); q.append(sql` WHERE id = ${1}`); read(q.text, q.values) // expect: Literal
}
export function conditionalTwo() {  const q = flag ? factory(sql`/* one */ SELECT `, extra).append(sql` WHERE id = ${1}`) : factory(sql`/* two */ SELECT `, extra).append(sql` WHERE id = ${2}`);  read(q.text, q.values) // expect: Literal
}
export function eightLeaves() {  const q = flag ? factory(sql`/* 6 */ SELECT `) : (flag ? factory(sql`/* 5 */ SELECT `) : (flag ? factory(sql`/* 4 */ SELECT `) : (flag ? factory(sql`/* 3 */ SELECT `) : (flag ? factory(sql`/* 2 */ SELECT `) : (flag ? factory(sql`/* 1 */ SELECT `) : (flag ? factory(sql`/* 0 */ SELECT `) : (factory(sql`/* eight */ SELECT `))))))));  read(q.text, q.values) // expect: Literal
}
export function bothMissing() {  const q = flag ? factory(sql`SELECT `) : factory(sql`SELECT `);  read(q.text, q.values) // expect: Missing
}
export function mixedMissing() {  const q = flag ? factory(sql`/* named */ SELECT `) : factory(sql`SELECT `);  read(q.text, q.values) // expect: Unproven
}
export function unknownArm() {  const q = flag ? factory(sql`/* named */ SELECT `) : other;  read(q.text, q.values) // expect: Unproven
}
export function differentFactory() {  const q = flag ? factory(sql`/* named */ SELECT `) : unproven(sql`/* other */ SELECT `);  read(q.text, q.values) // expect: Unproven
}
export function threeArguments() {  const q = factory(sql`/* three */ SELECT `, extra, sql` FROM other`);  read(q.text, q.values) // expect: Unproven
}
export function spread() {  const q = factory(sql`/* spread */ SELECT `, ...extra);  read(q.text, q.values) // expect: Unproven
}
export function optional() {  const q = factory?.(sql`/* optional */ SELECT `);  read(q.text, q.values) // expect: Unproven
}
export function wrongTag() {  const q = factory(custom`/* custom */ SELECT `);  read(q.text, q.values) // expect: Unproven
}
export function notFresh() {  const q = factory(sql`/* fresh */ SELECT `.append(sql`id`), extra);  read(q.text, q.values) // expect: Unproven
}
export function escaped() {  const q = factory(sql`/* named */ SELECT `); inspect(q); read(q.text, q.values) // expect: Unproven
}
export function getterCapturesResult() {  const q = factory(sql`/* getter */ SELECT `, { get columns() { return q }, join: sql`` });  read(q.text, q.values) // expect: Unproven
}
export function alias() { const q = factory(sql`/* named */ SELECT `); const alias = q; read(alias.text, alias.values) // expect: Unproven
}
export function textWrite() {  const q = factory(sql`/* named */ SELECT `); q.text = "SELECT 2"; read(q.text, q.values) // expect: Unproven
}
export function wrongValues() { const q = factory(sql`/* named */ SELECT `); read(q.text, other.values) // expect: Unproven
}
export function unknownExtraCapture() { const localExtra = { get columns() { return q }, join: sql`` }; const q = factory(sql`/* extra */ SELECT `, localExtra);  read(q.text, q.values) // expect: Unproven
}
export function testTouchesResult() {  const q = inspect(q) ? factory(sql`/* named */ SELECT `) : factory(sql`/* named */ SELECT `);  read(q.text, q.values) // expect: Unproven
}
export function missingValues() { const q = factory(sql`/* named */ SELECT `); read(q.text) // expect: Unproven
}
export function overBudget() {  const q = flag ? factory(sql`/* ninth */ SELECT `) : (flag ? factory(sql`/* 6 */ SELECT `) : (flag ? factory(sql`/* 5 */ SELECT `) : (flag ? factory(sql`/* 4 */ SELECT `) : (flag ? factory(sql`/* 3 */ SELECT `) : (flag ? factory(sql`/* 2 */ SELECT `) : (flag ? factory(sql`/* 1 */ SELECT `) : (flag ? factory(sql`/* 0 */ SELECT `) : (factory(sql`/* eight */ SELECT `)))))))));  read(q.text, q.values) // expect: Unproven
}
export function beginRepair() {  const q = factory(sql`BEGIN`).append("NING");  read(q.text, q.values) // expect: Unproven
}
export function commentRepair() {  const q = factory(sql`/*`).append(" named */ SELECT 1");  read(q.text, q.values) // expect: Unproven
}
export function harmlessGetter() {  const q = factory(sql`/* getter */ SELECT `, { get columns() { return other }, join: sql`` });  read(q.text, q.values) // expect: Literal
}
export function sameUnprovenFactory() {  const q = flag ? unproven(sql`/* no */ SELECT `) : unproven(sql`/* no */ SELECT `);  read(q.text, q.values) // expect: Unproven
}
export function capturedAfter() {  const q = factory(sql`/* named */ SELECT `); inspect(() => q); read(q.text, q.values) // expect: Unproven
}
export function twoReads() { const q = flag ? factory(sql`/* named */ SELECT `) : factory(sql`/* named */ SELECT `); read(q.text, q.values); read(q.text, q.values) // expect: Literal,Literal
}
export function shadow(factory) { const q = flag ? factory(sql`/* one */ SELECT `) : factory(sql`/* two */ SELECT `); read(q.text, q.values) // expect: Unproven
}
export const publicQuery = flag ? factory(sql`/* public */ SELECT `) : factory(sql`/* public */ SELECT `)
read(publicQuery.text, publicQuery.values) // expect: Unproven
export function oneBadPairRejectsAll() {
  const q = flag ? factory(sql`/* paired */ SELECT `) : factory(sql`/* paired */ SELECT `)
  read(q.text, other.values) // expect: Unproven
  read(q.text, q.values) // expect: Unproven
}
export function nonfreshUpdateInput() {
  const input = sql`/* updateInput */ SELECT `
  const q = factory(input.append(sql` WHERE id = ${1}`), extra, sql` FROM other`)
  read(q.text, q.values) // expect: Unproven
}

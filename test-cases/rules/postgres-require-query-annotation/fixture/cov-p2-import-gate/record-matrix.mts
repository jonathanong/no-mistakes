import {read} from '@bench/db';
import sql from 'sql-template-strings';
const blockArrow = () => { const text = '/* arrow */ SELECT 1'; return {text, values: []}; };
function arrowCaller() { const q = blockArrow(); read(q.text, q.values); } // low: true
const bareArrow = () => { return; };
function bareCaller() { const q = bareArrow(); read(q.text, q.values); } // low: false
declare function declared(): {text: string, values: unknown[]};
function declaredCaller() { const q = declared(); read(q.text, q.values); } // low: false
function localText() { let text = '/* local */ SELECT 1'; text = runtimeText; return {text, values: []}; }
function localCaller() { const q = localText(); read(q.text, q.values); } // low: false
function capturedText() { const text = '/* captured */ SELECT 1'; function inner() { return {text, values: []}; } const q = inner(); read(q.text, q.values); } // low: false
function named() { return {text: '/* named */ SELECT 1', values: []}; }
function typedResult() { const q = named(); type Result = typeof q; read(q.text, q.values); } // low: false
function returnedBuilder() { const q = sql`/* returned */ SELECT 1`; return q; }
function builderCaller() { const q = returnedBuilder(); read(q.text, q.values); } // low: true
function typeFactory(flag) { if(flag) return '/* first */ SELECT 1'; return '/* second */ SELECT 2'; }
type Factory = typeof typeFactory;
read(typeFactory(true)); // low: false
function defaultBare(value) { switch(value) { case 1: return sql`/* case */ SELECT 1`; default: return; } }
read(defaultBare(1)); // low: false
function caseBare(value) { switch(value) { case 1: return; default: throw Error('invalid'); } }
read(caseBare(1)); // low: false
function alternateBare(flag) { if(flag) return sql`/* first */ SELECT 1`; else return; return sql`/* second */ SELECT 2`; }
read(alternateBare(true)); // low: false
function mixedKinds(flag) { if(flag) return sql`/* builder */ SELECT 1`; return '/* primitive */ SELECT 2'; }
read(mixedKinds(true)); // low: true
function onlyThrows() { throw Error('invalid'); }
function thrownCaller() { const q = onlyThrows(); read(q); } // low: false

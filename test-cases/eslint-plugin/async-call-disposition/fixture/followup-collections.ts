import { enqueueEmail } from "@app/jobs";
async function race() { const ps = []; ps.push(enqueueEmail("race")); await Promise.race(ps); }
async function any() { const ps = []; ps.push(enqueueEmail("any")); return Promise.any(ps); }
async function mapped(xs) { const ps = xs.map(x => enqueueEmail(x)); await Promise.all(ps); }
async function mappedBlock(xs) { const ps = xs.map(x => { return enqueueEmail(x); }); await Promise.allSettled(ps); }
async function each(xs) { const ps = []; xs.forEach(x => { ps.push(enqueueEmail(x)); }); await Promise.all(ps); }
async function finalized() { const ps = []; try { ps.push(enqueueEmail("finally")); await run(); } finally { await Promise.all(ps); } }
async function beforeTry() { const ps = []; ps.push(enqueueEmail("before")); try { return await run(); } finally { await Promise.all(ps); } }
async function eachExpression(xs) { const ps = []; xs.forEach(x => ps.push(enqueueEmail(x))); await Promise.all(ps); }
async function returnFromTry() { const ps = []; try { ps.push(enqueueEmail("return")); return await run(); } finally { await Promise.all(ps); } }
async function throwFromTry() { const ps = []; try { ps.push(enqueueEmail("throw")); throw error; } finally { await Promise.all(ps); } }
// This previously reported fixture is now a supported retained map collection.
async function formerlyInvalid(items) {
  const callbacks = items.map((item) => {
    return enqueueEmail(item);
  });
  await Promise.all(callbacks);
}
async function branchedReturn() { const p = enqueueEmail("branches"); if (cond) { own(p); } else { inspect(p); } return p; }
// Retained map arrays use the same guaranteed finally drain as pushed arrays.
async function mappedFinally(xs) { const ps = xs.map(x => enqueueEmail(x)); try { return await run(); } finally { await Promise.all(ps); } }
async function returnedEachFinally(xs) { const ps = []; try { return xs.forEach(x => ps.push(enqueueEmail(x))); } finally { await Promise.all(ps); } }
async function thrownEachFinally(xs) { const ps = []; try { throw xs.forEach(x => ps.push(enqueueEmail(x))); } finally { await Promise.all(ps); } }

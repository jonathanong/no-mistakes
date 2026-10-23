import { enqueueEmail } from "@app/jobs";
import { vi } from "vitest";
async function awaitArray(xs) { const ps = xs.map(x => enqueueEmail(x)); await ps; }
function voidArray(xs) { const ps = xs.map(x => enqueueEmail(x)); void ps; }
async function escapedAfter(xs) { const ps = xs.map(x => enqueueEmail(x)); await Promise.all(ps); sink(ps); }
async function earlyEach(xs) { const ps = []; xs.forEach(x => { if (cond) return; ps.push(enqueueEmail(x)); }); await Promise.all(ps); }
async function earlyFinally() { const ps = []; ps.push(enqueueEmail("early")); if (cond) return; try { await run(); } finally { await Promise.all(ps); } }
async function beforeFinallyReturn() { const ps = []; try { ps.push(enqueueEmail("early")); } finally { if (cond) return; await Promise.all(ps); } }
function rewrittenMocked() { const mock = vi.mocked(service.send); mock.mockImplementation = unknown; mock.mockImplementation(x => enqueueEmail(x)); }
function rewrittenLet() { let spy = vi.spyOn(service, "send"); let alias = spy; alias.mockImplementation = unknown; spy.mockImplementation(x => enqueueEmail(x)); }
function conditionalExit() { const p = enqueueEmail("early"); if (cond) { if (other) return; } return p; }
function secondResolveArg() { return Promise.resolve(other, enqueueEmail("ignored")); }
async function asyncExpression(xs) { const ps = []; xs.forEach(async x => ps.push(enqueueEmail(x))); await Promise.all(ps); }
async function outsideAttempt() { const ps = []; { ps.push(enqueueEmail("outer")); } try { run(); } finally { await Promise.all(ps); } }
async function conditionalEach(xs) { const ps = []; if (cond) xs.forEach(x => ps.push(enqueueEmail(x))); await Promise.all(ps); }
async function logicalEach(xs) { const ps = []; cond && xs.forEach(x => ps.push(enqueueEmail(x))); await Promise.all(ps); }
async function mappedConditionalFinally(xs) { const ps = xs.map(x => enqueueEmail(x)); try { run(); } finally { if (cond) await Promise.all(ps); } }
async function mappedCatchOnly(xs) { const ps = xs.map(x => enqueueEmail(x)); try { run(); } catch { await Promise.all(ps); } }
async function returnedEach(xs) { const ps = []; return xs.forEach(x => ps.push(enqueueEmail(x))); await Promise.all(ps); }
async function thrownEach(xs) { const ps = []; throw xs.forEach(x => ps.push(enqueueEmail(x))); await Promise.all(ps); }
async function returnedBeforeFinally(xs) { const ps = []; return xs.forEach(x => ps.push(enqueueEmail(x))); try { run(); } finally { await Promise.all(ps); } }

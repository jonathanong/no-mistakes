import { enqueueEmail } from "@app/jobs";
import { vi } from "vitest";
async function never(xs) { const ps = xs.map(x => enqueueEmail(x)); }
async function escaped(xs) { const ps = xs.map(x => enqueueEmail(x)); sink(ps); await Promise.all(ps); }
function returned(xs) { const ps = xs.map(x => enqueueEmail(x)); return ps; }
function floating(xs) { const ps = xs.map(x => enqueueEmail(x)); Promise.all(ps); }
async function late(xs) { const ps = []; xs.forEach(async x => { await wait(); ps.push(enqueueEmail(x)); }); await Promise.all(ps); }
async function unknown(xs) { const ps = []; iterate(xs, x => { ps.push(enqueueEmail(x)); }); await Promise.all(ps); }
async function conditional(xs) { const ps = []; xs.forEach(x => { if (cond) ps.push(enqueueEmail(x)); }); await Promise.all(ps); }
async function catchOnly() { const ps = []; try { ps.push(enqueueEmail("catch")); } catch { await Promise.all(ps); } }
async function guardedFinally() { const ps = []; try { ps.push(enqueueEmail("guard")); } finally { if (cond) await Promise.all(ps); } }
async function floatingFinally() { const ps = []; try { ps.push(enqueueEmail("float")); } finally { Promise.all(ps); } }
async function escapedFinally() { const ps = []; try { ps.push(enqueueEmail("escape")); sink(ps); } finally { await Promise.all(ps); } }
function early() { const p = enqueueEmail("early"); if (cond) return; return p; }
function thrown() { const p = enqueueEmail("throw"); if (cond) throw error; return p; }
function custom(Promise) { return Promise.resolve(enqueueEmail("shadow")); }
function wrapper() { return wrap(enqueueEmail("unknown")); }
function reassigned() { let spy = vi.fn(); spy = unknown; spy.mockImplementation(x => enqueueEmail(x)); }
function shadowed(vi) { vi.mocked(service.send).mockImplementation(x => enqueueEmail(x)); }
async function shadowPromise(Promise) { const ps = []; ps.push(enqueueEmail("shadow")); await Promise.race(ps); }
async function mutation() { const ps = []; ps.push(enqueueEmail("mutate")); ps.length = 0; await Promise.any(ps); }
async function lateDrain(xs) { const ps = xs.map(x => enqueueEmail(x)); if (cond) return; await Promise.all(ps); }
async function nestedDrain(xs) { const ps = xs.map(x => enqueueEmail(x)); function later() { return Promise.all(ps); } }

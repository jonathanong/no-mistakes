import { enqueueEmail } from "@app/jobs";
import { vi } from "vitest";

// Lookalike methods and other argument positions do not promise pass-through.
other.mockImplementation(() => enqueueEmail("1"));
unknown(() => enqueueEmail("1"));
vi.fn(undefined, () => enqueueEmail("1"));
function shadowed(vi) { vi.fn(() => enqueueEmail("1")); }
function fakePromise(Promise) { const pending = []; pending.push(enqueueEmail("1")); return Promise.all(pending); }
function neverDrained() { const pending = []; pending.push(enqueueEmail("1")); }
function floatingDrain() { const pending = []; pending.push(enqueueEmail("1")); Promise.all(pending); }
async function wrongArray() { const pending = []; const other = []; pending.push(enqueueEmail("1")); await Promise.all(other); }
async function beforePush() { const pending = []; await Promise.all(pending); pending.push(enqueueEmail("1")); }
async function futureCallback() { const pending = []; unknown(() => { pending.push(enqueueEmail("1")); }); await Promise.all(pending); }
async function mutated() { const pending = []; pending.push(enqueueEmail("1")); pending.pop(); await Promise.all(pending); }
async function conditionalDrain(flag) { const pending = []; pending.push(enqueueEmail("1")); if (flag) await Promise.all(pending); }
function conditionalReturn(flag) { const job = enqueueEmail("1"); if (flag) return job; }
async function overwritten() { let job = enqueueEmail("1"); job = other(); await job; }
// A later function can run after its parent exits; capturing a promise is not disposition.
function retainedClosure() { const job = enqueueEmail("1"); const later = () => job; }
function guardedReturn(flag) { const job = enqueueEmail("1"); flag && void job; }
async function guardedDrain(flag) { const pending = []; pending.push(enqueueEmail("1")); flag && await Promise.all(pending); }
unknown((() => { const job = enqueueEmail("1"); return job; }) as Impl);
async function ternaryDrain(flag) { const pending = []; pending.push(enqueueEmail("1")); flag ? await Promise.all(pending) : null; }
async function optionalDrain() { const pending = []; pending.push(enqueueEmail("1")); void Promise.all?.(pending); }
function localLookalike() { const vi = fake; vi.fn(() => enqueueEmail("1")); }
function namespaceLookalike() { const vi = fake; vi.other.fn(() => enqueueEmail("1")); }
async function escaped() { const pending = []; pending.push(enqueueEmail("1")); mutate(pending); await Promise.all(pending); }
async function nonempty() { const pending = [other]; pending.push(enqueueEmail("1")); await Promise.all(pending); }
async function mutableArray() { let pending = []; pending.push(enqueueEmail("1")); await Promise.all(pending); }
async function controlBetween(flag) { const pending = []; pending.push(enqueueEmail("1")); if (flag) return; await Promise.all(pending); }
unknown(() => { const job = enqueueEmail("1"); return job; });
function droppedValue() { const job = enqueueEmail("1"); }
function unstableMock() { let mock = vi.fn(); mock = other; mock.mockImplementation(() => enqueueEmail("1")); }
import { vi as unrelatedVi } from "other";
import type { vi as typeVi } from "vitest";
import { type vi as specifierTypeVi } from "vitest";
import type * as typeVitest from "vitest";
unrelatedVi.fn(() => enqueueEmail("1"));
typeVi.fn(() => enqueueEmail("1"));
specifierTypeVi.fn(() => enqueueEmail("1"));
typeVitest.vi.fn(() => enqueueEmail("1"));
otherFactory().mockImplementation(() => enqueueEmail("1"));
// A known factory cannot prove a rewritten implementation method's return contract.
function rewrittenMock() { const mock = vi.fn(); mock.mockImplementation = unknown; mock.mockImplementation(() => enqueueEmail("1")); }
function deletedMock() { const mock = vi.fn(); delete mock.mockImplementation; mock.mockImplementation(() => enqueueEmail("1")); }
function updatedMock() { const mock = vi.fn(); mock.mockImplementation++; mock.mockImplementation(() => enqueueEmail("1")); }
function aliasMutation() { const mock = vi.fn(); const alias = mock; alias.mockImplementation = unknown; mock.mockImplementation(() => enqueueEmail("1")); }
function rewrittenFramework() { const api = vi; api.fn = unknown; api.fn(() => enqueueEmail("1")); }

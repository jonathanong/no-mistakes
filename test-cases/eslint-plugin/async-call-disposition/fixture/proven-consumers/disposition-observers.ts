import { expect } from 'vitest';
import { enqueueEmail } from '@app/jobs';
async function consume(callback: () => Promise<unknown>) { await callback(); }
function unknown(callback: () => Promise<unknown>) { console.log(callback); }
async function cases() {
  await expect(enqueueEmail()).rejects.toThrow();
  await expect(enqueueEmail()).resolves.toBeDefined();
  const retained = enqueueEmail();
  await expect(retained).resolves.toBeDefined();
  enqueueEmail().then(() => {}).catch(() => {});
  consume(() => enqueueEmail());
  unknown(() => enqueueEmail()); // Unknown consumers must keep reporting.
  expect(enqueueEmail()).rejects.toThrow();
  enqueueEmail().catch();
  enqueueEmail().catch(undefined);
  const alias = expect;
  await alias(enqueueEmail()).rejects.toThrow();
}
async function shadow(expect: any) { await expect(enqueueEmail()).rejects.toThrow(); }
async function unknownExpect(check: any) { await check(enqueueEmail()).rejects.toThrow(); }
enqueueEmail().catch(() => { enqueueEmail(); }); // Handler promises remain independently checked.
function onRejected() {}
enqueueEmail().catch(onRejected);
const stableHandler = () => {};
enqueueEmail().catch(stableHandler);
let mutableHandler = () => {};
mutableHandler = () => {};
enqueueEmail().catch(mutableHandler);
enqueueEmail().catch(42);
enqueueEmail().catch(...[onRejected]);
await expect(enqueueEmail()).toBeDefined();
await expect(enqueueEmail());
await expect(enqueueEmail(), 42).rejects.toThrow();
await expect(enqueueEmail()).rejects;
await expect(enqueueEmail()).rejects.toThrow;
async function localExpect() {
  const expect = (value: unknown): any => value;
  await expect(enqueueEmail()).rejects.toThrow();
}
enqueueEmail().unknownMethod().catch(() => {}); // Unknown member consumers may discard the promise.
enqueueEmail().then(() => {}).unknownMethod().catch(() => {});
enqueueEmail().catch(() => {}).finally(() => {}).catch(onRejected);
enqueueEmail().catch(unknownHandler);
enqueueEmail().catch?.(onRejected);
await expect(enqueueEmail()).resolves.unknown?.();
await expect(enqueueEmail()).resolves?.unknown();
const retainedUnknown = enqueueEmail().unknownMethod();
retainedUnknown.catch(() => {});
const retainedKnown = enqueueEmail().then(() => {});
retainedKnown.catch(() => {});

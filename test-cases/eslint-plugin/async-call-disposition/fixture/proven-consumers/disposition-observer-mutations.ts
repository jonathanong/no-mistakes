import { enqueueEmail } from '@app/jobs';
async function directCatch() {
  const p = enqueueEmail();
  p.catch = () => {};
  p.catch(() => {});
}
async function aliasedCatch() {
  const p = enqueueEmail();
  const alias = p;
  alias.catch = () => {};
  p.catch(() => {});
}
async function reflectedCatch() {
  const p = enqueueEmail();
  Object.defineProperty(p, 'catch', { value: () => {} });
  p.catch(() => {});
}
async function fakeExpect() {
  globalThis.expect = () => ({ rejects: { toThrow() {} } });
  const p = enqueueEmail();
  await expect(p).rejects.toThrow();
}
const holder = globalThis;
Object.defineProperty(holder, 'expect', { value: () => ({}) });
const promiseAlias = Promise;
Reflect.set(promiseAlias.prototype, 'catch', () => {});
async function mutatedPromise() {
  enqueueEmail().catch(() => {});
}
async function deletedCatch() {
  const p = enqueueEmail();
  delete p.catch;
  p.catch(() => {});
}
async function updatedCatch() {
  const p = enqueueEmail();
  p.catch++;
  p.catch(() => {});
}
async function dynamicCatch(key: string) {
  const p = enqueueEmail();
  p[key] = () => {};
  p.catch(() => {});
}

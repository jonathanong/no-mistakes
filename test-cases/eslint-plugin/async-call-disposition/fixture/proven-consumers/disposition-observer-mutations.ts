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
async function mutableReflectAlias() {
  const p = enqueueEmail();
  let alias = p;
  Reflect.set(alias, 'catch', () => {});
  p.catch(() => {});
}
async function reflectFunctionAlias() {
  const p = enqueueEmail();
  const change = Reflect.set;
  change(p, 'catch', () => {});
  p.catch(() => {});
}
async function assignedCycleAlias() {
  const p = enqueueEmail();
  let first;
  let second = first;
  first = p;
  second = first;
  first = second;
  const change = Reflect.set;
  change(second, 'catch', () => {});
  p.catch(() => {});
}
let mutableHolder = globalThis;
const changeHolder = Reflect.set;
changeHolder(mutableHolder, 'expect', () => ({}));
async function unknownReflectAlias(change: any) {
  const p = enqueueEmail();
  const mutate = change;
  mutate(p, 'catch', () => {});
  p.catch(() => {});
}
async function unknownReflectionObject(api: any) {
  const p = enqueueEmail();
  const mutate = api.set;
  mutate(p, 'irrelevant', () => {});
  p.catch(() => {});
}
async function opaqueHolder() {
  const p = enqueueEmail();
  const box = { p };
  Reflect.set(box.p, 'catch', () => {});
  p.catch(() => {});
}
async function opaqueArray() {
  const p = enqueueEmail();
  const box = [p];
  Reflect.set(box[0], 'catch', () => {});
  p.catch(() => {});
}
async function resolvedIdentity() {
  const p = enqueueEmail();
  const same = Promise.resolve(p);
  p.catch = () => {};
  same.catch(() => {});
}
async function chainIdentity() {
  const p = enqueueEmail();
  const chain = p.then(() => {});
  chain.catch = () => {};
  chain.catch(() => {});
}

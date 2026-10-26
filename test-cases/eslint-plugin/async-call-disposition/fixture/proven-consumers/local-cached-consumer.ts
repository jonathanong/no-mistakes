import { enqueueEmail } from "@app/jobs";

declare function unknownConsumer(callback: () => Promise<unknown>): Promise<unknown>;
declare function sideEffect(value?: unknown): void;

async function safe(callback: () => Promise<unknown>) {
  const cache = new Map<string, Promise<unknown>>();
  if (!cache.has('k')) cache.set('k', callback());
  return await cache.get('k');
}
const safeJob = () => enqueueEmail(1); // accepted
await safe(safeJob);

async function wrongKey(callback: () => Promise<unknown>) {
  const cache = new Map<string, Promise<unknown>>();
  if (!cache.has('k')) cache.set('k', callback());
  return await cache.get('other');
}
const wrongKeyJob = () => enqueueEmail(1); // report
await wrongKey(wrongKeyJob);

async function bareReturn(callback: () => Promise<unknown>) {
  const cache = new Map<string, Promise<unknown>>();
  if (!cache.has('k')) cache.set('k', callback());
  return cache.get('k');
}
const bareReturnJob = () => enqueueEmail(1); // report
await bareReturn(bareReturnJob);

async function overwrite(callback: () => Promise<unknown>) {
  const cache = new Map<string, Promise<unknown>>();
  if (!cache.has('k')) cache.set('k', callback());
  cache.set('k', Promise.resolve());
  return await cache.get('k');
}
const overwriteJob = () => enqueueEmail(1); // report
await overwrite(overwriteJob);

async function throws(callback: () => Promise<unknown>) {
  const cache = new Map<string, Promise<unknown>>();
  if (!cache.has('k')) cache.set('k', callback());
  sideEffect();
  return await cache.get('k');
}
const throwsJob = () => enqueueEmail(1); // report
await throws(throwsJob);

async function alias(callback: () => Promise<unknown>) {
  const cache = new Map<string, Promise<unknown>>();
  const alias = cache;
  if (!cache.has('k')) alias.set('k', callback());
  return await cache.get('k');
}
const aliasJob = () => enqueueEmail(1); // report
await alias(aliasJob);

async function escapes(callback: () => Promise<unknown>) {
  const cache = new Map<string, Promise<unknown>>();
  if (!cache.has('k')) cache.set('k', callback());
  sideEffect(cache);
  return await cache.get('k');
}
const escapesJob = () => enqueueEmail(1); // report
await escapes(escapesJob);

async function wrongValue(callback: () => Promise<unknown>) {
  const cache = new Map<string, Promise<unknown>>();
  if (!cache.has('k')) cache.set('k', (callback(), Promise.resolve()));
  return await cache.get('k');
}
const wrongValueJob = () => enqueueEmail(1); // report
await wrongValue(wrongValueJob);

async function mutation(callback: () => Promise<unknown>) {
  const cache = new Map<string, Promise<unknown>>();
  if (!cache.has('k')) cache.set('k', callback());
  cache.clear();
  return await cache.get('k');
}
const mutationJob = () => enqueueEmail(1); // report
await mutation(mutationJob);

 {
const Map = globalThis.Map; // A local constructor cannot establish the built-in contract.
async function shadowed(callback: () => Promise<unknown>) {
  const cache = new Map<string, Promise<unknown>>();
  if (!cache.has('k')) cache.set('k', callback());
  return await cache.get('k');
}
const shadowedJob = () => enqueueEmail(1); // report
await shadowed(shadowedJob);

}
async function unawaited(callback: () => Promise<unknown>) {
  const cache = new Map<string, Promise<unknown>>();
  if (!cache.has('k')) cache.set('k', callback());
  cache.get('k');
}
const unawaitedJob = () => enqueueEmail(1); // report
await unawaited(unawaitedJob);

const unknownJob = () => enqueueEmail(1); // report
await unknownConsumer(unknownJob);

async function braced(callback: () => Promise<unknown>) {
  const cache = new Map();
  if (!cache.has("k")) { cache.set("k", callback()); }
  return await cache.get("k");
}
const bracedJob = () => enqueueEmail(1); // accepted
await braced(bracedJob);

async function wrongGuard(callback: () => Promise<unknown>) {
  const cache = new Map();
  if (!cache.has("other")) cache.set("k", callback());
  return await cache.get("k");
}
const wrongGuardJob = () => enqueueEmail(1); // report
await wrongGuard(wrongGuardJob);

async function seeded(callback: () => Promise<unknown>) {
  const cache = new Map([["k", Promise.resolve()]]);
  if (!cache.has("k")) cache.set("k", callback());
  return await cache.get("k");
}
const seededJob = () => enqueueEmail(1); // report
await seeded(seededJob);

async function argumentEffect(callback: () => Promise<unknown>) {
  const cache = new Map();
  if (!cache.has("k")) cache.set("k", callback(sideEffect()));
  return await cache.get("k");
}
const argumentEffectJob = () => enqueueEmail(1); // report
await argumentEffect(argumentEffectJob);

async function ignoredRead(callback: () => Promise<unknown>) {
  const cache = new Map();
  if (!cache.has("k")) cache.set("k", callback());
  return void cache.get("k");
}
const ignoredReadJob = () => enqueueEmail(1); // report
await ignoredRead(ignoredReadJob);

async function defaulted(callback: () => Promise<unknown>, value = sideEffect()) {
  const cache = new Map();
  if (!cache.has("k")) cache.set("k", callback());
  return await cache.get("k");
}
const defaultedJob = () => enqueueEmail(1); // report
await defaulted(defaultedJob);

async function elseBranch(callback: () => Promise<unknown>) {
  const cache = new Map();
  if (!cache.has("k")) cache.set("k", callback()); else sideEffect();
  return await cache.get("k");
}
const elseBranchJob = () => enqueueEmail(1); // report
await elseBranch(elseBranchJob);

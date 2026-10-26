import { enqueueEmail } from "@app/jobs";
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

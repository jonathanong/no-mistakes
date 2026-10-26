import { enqueueEmail } from "@app/jobs";

// Source-visible mutation invalidates the built-in cache contract.
Map = class extends Map {};
async function cached(callback: () => Promise<unknown>) {
  const cache = new Map();
  if (!cache.has("k")) cache.set("k", callback());
  return await cache.get("k");
}
const job = () => enqueueEmail(1); // report
await cached(job);

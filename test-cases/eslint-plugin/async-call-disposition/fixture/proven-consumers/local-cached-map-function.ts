import { enqueueEmail } from "@app/jobs";
// Dynamic code can replace Map methods without a direct Map reference.
Function("Map.prototype.get = () => undefined")();
async function cached(callback: () => Promise<unknown>) {
  const cache = new Map();
  if (!cache.has("k")) cache.set("k", callback());
  return await cache.get("k");
}
const job = () => enqueueEmail(1); // report
await cached(job);

import { enqueueEmail } from "@app/jobs";
// An opaque global key can select Map; do not prove through prototype replacement.
const selected = "Map";
Reflect.set(globalThis[selected].prototype, "get", () => undefined);
async function cached(callback: () => Promise<unknown>) {
  const cache = new Map();
  if (!cache.has("k")) cache.set("k", callback());
  return await cache.get("k");
}
const job = () => enqueueEmail(1); // report
await cached(job);

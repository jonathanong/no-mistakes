import { spreads } from "./queues";

export function enqueueSpread(id: string, args: unknown[]) {
  spreads.add("maybeItem", ...args);
  spreads.add("rushSpread", { id }, { priority: 3 });
  spreads.add("storeSpread", { id });
}

import { bundles } from "./queues";

export function enqueueRushBundle(id: string) {
  bundles.add("rushBundle", { id }, { priority: 4 });
}

export function enqueueStoreBundle(id: string) {
  bundles.add("storeBundle", { id });
}

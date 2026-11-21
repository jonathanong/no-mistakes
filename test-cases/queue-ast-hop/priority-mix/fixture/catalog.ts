import { enqueueWidget } from "./lib";
import { catalog } from "./queues";

export function enqueuePlainCatalog() {
  return enqueueWidget(catalog, "listItem");
}

export function enqueueRushCatalog() {
  return enqueueWidget(catalog, "rushItem", { priority: 8 });
}

export function enqueueFastCatalog() {
  return enqueueWidget(catalog, "fastItem", { priority: 1 });
}

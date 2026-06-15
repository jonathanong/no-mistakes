import { ValkeyCache } from "valkey";

export function makeCache() {
  const cache = new ValkeyCache();
  return getEntityCache(cache);
}

function getEntityCache(cache: unknown) {
  return cache;
}

import { loadDateFormatter } from "./dynamic-import-unused.mts";

export async function testDynamicImportUnused() {
  await loadDateFormatter(new Date("2026-01-01"));
}

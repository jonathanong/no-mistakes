import { loadDateDynamically } from "./dynamic-import-caller.mts";

export async function testDynamicImportCaller() {
  await loadDateDynamically("2026-01-01");
}

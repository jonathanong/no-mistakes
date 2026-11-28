import { vi } from "vitest";

const name = "dynamic";

export async function loadDirect() {
  const computed = await vi.importActual(name);
  const samePackage = await vi.importMock("./a.ts");
  const missing = await vi.importActual("../../shared/src/missing-topic.mts");
  const cross = await vi.importMock("../../shared/src/status.mts");
  const packageName = await vi.importActual("@sample/shared");
  return { computed, samePackage, missing, cross, packageName };
}

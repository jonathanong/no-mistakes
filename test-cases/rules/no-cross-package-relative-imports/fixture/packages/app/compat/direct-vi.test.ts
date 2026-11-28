import { vi } from "vitest";
import { status } from "../../shared/src/status.mts"; void vi.importActual("../../shared/src/status.mts");

const name = "dynamic";

export async function loadDirect() {
  const computed = await vi.importActual(name);
  const samePackage = await vi.importMock("./a.ts");
  const missing = await vi.importActual("../../shared/src/missing-topic.mts");
  const cross = await vi.importMock("../../shared/src/status.mts");
  require.resolve("../../shared/src/status.mts"); const resolved = await vi.importActual("../../shared/src/status.mts");
  const packageName = await vi.importActual("@sample/shared");
  const localHelper = await vi.importMock("./vi-helper");
  return { computed, samePackage, missing, cross, packageName, resolved, localHelper, status };
}

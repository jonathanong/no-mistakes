import { vi as mock } from "vitest";
import * as vitest from "vitest";
import sharedWav from "../../shared/fixtures/tone.wav";

export const tone = sharedWav;

async function loadCrossPackageFixtures() {
  const widget = await mock.importActual<typeof import("../../shared/src/widget")>(
    "../../shared/src/widget",
  );
  const status = await vitest.vi.importMock("../../shared/src/status.mts");
  return { widget, status };
}

export { loadCrossPackageFixtures };

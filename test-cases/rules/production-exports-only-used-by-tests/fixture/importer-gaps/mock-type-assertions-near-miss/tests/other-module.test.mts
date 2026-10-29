// The asserted module differs from the mocked module.
import { vi } from "vitest";

vi.mock("../src/api.mjs", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../src/other-module.mjs")>()),
}));

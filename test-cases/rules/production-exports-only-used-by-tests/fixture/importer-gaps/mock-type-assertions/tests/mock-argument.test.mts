import { vi } from "vitest";

vi.mock("../src/api.mjs", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../src/api.mjs")>()),
}));

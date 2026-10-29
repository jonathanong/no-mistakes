import { vi } from "vitest";

vi.mock("../src/api.mjs", async function (importOriginal) {
  return { ...(await importOriginal<typeof import("../src/api.mjs")>()) };
});

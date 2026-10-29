import { vi } from "vitest";

vi.mock("../src/api.mjs", async (importOriginal) => {
  const actual = (await importOriginal()) as typeof import("../src/api.mjs");
  return { ...actual };
});

import { expect, it, vi } from "vitest";

vi.mock("../src/feature", () => ({ feature: "mocked" }));

it("loads the feature", async () => {
  const { loadFeature } = await import("../src/lazy");
  await expect(loadFeature()).resolves.toEqual({ feature: "mocked" });
});

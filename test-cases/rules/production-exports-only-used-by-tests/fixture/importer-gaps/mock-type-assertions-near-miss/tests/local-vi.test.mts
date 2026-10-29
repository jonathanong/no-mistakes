// A local vi object is not the Vitest mock utility.
const vi = { mock: (_path: string, _factory: unknown) => undefined };

vi.mock("../src/local-vi.mjs", async (importOriginal: () => unknown) => ({
  ...((await importOriginal()) as typeof import("../src/local-vi.mjs")),
}));

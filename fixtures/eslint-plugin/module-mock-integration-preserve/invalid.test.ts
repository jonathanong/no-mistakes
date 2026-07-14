import { vi } from "vitest";

vi.mock("@app/wrong-specifier", async () => ({
  ...(await vi.importActual("@app/different")),
  taggedProviderCall: vi.fn(),
}));

vi.mock("@app/opaque-spread", () => ({
  ...unresolvedActual,
  taggedProviderCall: vi.fn(),
}));

// A real-module spread does not grant permission to explicit unmarked overrides.
vi.mock("@app/untagged", async () => ({
  ...(await vi.importActual("@app/untagged")),
  untaggedProviderCall: vi.fn(),
}));

vi.mock("@app/computed", async () => ({
  ...(await vi.importActual("@app/computed")),
  [exportName]: vi.fn(),
}));

vi.mock("@app/opaque-factory", () => makeMock());

vi.mock("@app/shadowed-framework", async () => {
  const vi = fakeFramework;
  return {
    ...(await vi.importActual("@app/shadowed-framework")),
    taggedProviderCall: vi.fn(),
  };
});

vi.mock("@app/shadowed-parameter", async (importOriginal) => {
  {
    const importOriginal = fakeLoader;
    return {
      ...(await importOriginal()),
      taggedProviderCall: vi.fn(),
    };
  }
});

vi.mock("@app/switch-return", async () => {
  switch (mode) {
    case "opaque":
      return makeMock();
    default:
      return {
        ...(await vi.importActual("@app/switch-return")),
        taggedProviderCall: vi.fn(),
      };
  }
});

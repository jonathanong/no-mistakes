import { vi } from "vitest";

vi.mock("./valid-inline", async () => ({
  ...(await vi.importActual("./valid-inline")),
  helper: vi.fn(),
}));

vi.mock("./valid-param", async importOriginal => ({
  ...(await importOriginal()),
  helper: vi.fn(),
}));

vi.mock("./valid-const", async () => {
  const actual = await vi.importActual("./valid-const");
  return { ...actual, helper: vi.fn() };
});

jest.mock("./valid-jest", () => ({
  ...jest.requireActual("./valid-jest"),
  helper: jest.fn(),
}));

vi.mock("./valid-spy", { spy: true });

vi.mock("./invalid-opaque", () => helperFactory());
vi.mock("./invalid-partial", () => ({ helper: vi.fn() }));
vi.mock("./invalid-other", async () => ({
  ...(await vi.importActual("./different")),
  helper: vi.fn(),
}));

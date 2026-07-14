import { jest } from "@jest/globals";
import { mock as importedMock, vi } from "vitest";

vi.mock("@app/inline", async () => ({
  ...(await vi.importActual("@app/inline")),
  taggedProviderCall: vi.fn(),
}));

vi.mock("@app/parameter", async (importOriginal) => ({
  ...(await importOriginal()),
  taggedProviderCall: vi.fn(),
}));

vi.mock("@app/const-alias", async () => {
  const actual = await vi.importActual("@app/const-alias");
  return { ...actual, taggedProviderCall: vi.fn() };
});

jest.mock("@app/jest", () => ({
  ...jest.requireActual("@app/jest"),
  taggedProviderCall: jest.fn(),
}));

importedMock("@app/imported", async (importOriginal) => ({
  ...(await importOriginal()),
  taggedProviderCall: vi.fn(),
}));

const mockAlias = vi.mock;
mockAlias.call(undefined, "@app/alias-call", async () => ({
  ...(await vi.importActual("@app/alias-call")),
  taggedProviderCall: vi.fn(),
}));
mockAlias.apply(undefined, [
  "@app/alias-apply",
  async () => ({
    ...(await vi.importActual("@app/alias-apply")),
    taggedProviderCall: vi.fn(),
  }),
]);

vi.mock.call(undefined, "@app/direct-call", async () => ({
  ...(await vi.importActual("@app/direct-call")),
  taggedProviderCall: vi.fn(),
}));
vi.mock.apply(undefined, [
  "@app/direct-apply",
  async () => ({
    ...(await vi.importActual("@app/direct-apply")),
    taggedProviderCall: vi.fn(),
  }),
]);

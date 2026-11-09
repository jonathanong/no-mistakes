import { vi } from "vitest";
const target = "../src/factory.mjs";
vi.doMock(import(target), (actual) => actual());

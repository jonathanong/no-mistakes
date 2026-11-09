import { vi } from "vitest";
const target = "../src/isolated.mjs";
vi.doMock(import(target), () => ({ isolated: 0 }));
vi.mock(import(new URL("../src/isolated.mjs", import.meta.url).href), () => ({ isolated: 0 }));

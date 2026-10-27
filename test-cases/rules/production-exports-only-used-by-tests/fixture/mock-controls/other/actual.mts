import { vi } from "vitest";
// A factory that loads actual source cannot prove an isolated replacement.
vi.mock(import("../src/actual.mjs"), () => vi.importActual("../src/actual.mjs"));

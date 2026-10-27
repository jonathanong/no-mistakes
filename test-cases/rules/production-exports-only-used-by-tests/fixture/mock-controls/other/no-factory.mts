import { vi } from "vitest";
// Without a replacement factory, Vitest may load the actual module.
vi.mock(import("../src/no-factory.mjs"));

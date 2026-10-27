import { vi } from "vitest";
// Vitest supplies importOriginal to this factory, which can load the production source.
vi.mock(import("../src/original.mjs"), (original) => original());

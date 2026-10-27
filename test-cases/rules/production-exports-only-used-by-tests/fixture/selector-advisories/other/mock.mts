import { vi } from "vitest";
// Vitest accepts a literal import expression as a static module selector.
vi.mock(import("../src/mocked.mjs"), () => ({ mockedValue: 2 }));

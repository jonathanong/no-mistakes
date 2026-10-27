import { vi } from "vitest";
// Static object shape cannot hide opaque execution or an accessor.
vi.mock(import("../src/tagged.mjs"), () => ({ value: () => tag`replacement` }));

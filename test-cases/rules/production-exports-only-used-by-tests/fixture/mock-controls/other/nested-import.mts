import { vi } from "vitest";
// Static object shape cannot hide opaque execution or an accessor.
vi.mock(import("../src/nested-import.mjs"), () => ({ value: () => import("../src/nested-import.mjs") }));

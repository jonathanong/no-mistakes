import { vi } from "vitest";
// Static object shape cannot hide opaque execution or an accessor.
vi.mock(import("../src/opaque-call.mjs"), () => ({ value: () => opaqueLoader() }));

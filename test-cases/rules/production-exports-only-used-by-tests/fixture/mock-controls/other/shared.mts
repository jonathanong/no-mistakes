import { vi } from "vitest";
// The genuine dynamic import on the same line must retain its unknown selector.
vi.mock(import("../src/shared.mjs"), () => ({ sharedValue: 2 })); void import("../src/shared.mjs");

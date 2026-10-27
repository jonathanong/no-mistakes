import { vi } from "vitest";
// Opaque execution or a noncanonical binding must retain this real module load.
vi.mock(import("../src/reference.mjs"), replacementFactory);

import { expect as vi } from "vitest";
// Opaque execution or a noncanonical binding must retain this real module load.
vi.mock(import("../src/wrong-binding.mjs"), () => ({ value: 2 }));

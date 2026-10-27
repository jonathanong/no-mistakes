import { vi } from "./fake.mjs";
// Opaque execution or a noncanonical binding must retain this real module load.
vi.mock(import("../src/local-binding.mjs"), () => ({ value: 2 }));

// No imported binding identifies this global name as Vitest.
// Opaque execution or a noncanonical binding must retain this real module load.
vi.mock(import("../src/unbound.mjs"), () => ({ value: 2 }));

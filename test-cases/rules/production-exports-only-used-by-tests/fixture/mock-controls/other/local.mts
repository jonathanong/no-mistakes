// A local object may consume the module promise; it is not Vitest.
const vi = { mock: (module: unknown, factory: unknown) => void [module, factory] };
vi.mock(import("../src/local.mjs"), () => ({ localValue: 2 }));

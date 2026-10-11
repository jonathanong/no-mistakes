import { vi } from 'vitest';
export type Shape = unknown;
vi.mock(import('../src/type-only-leaf.mts'), () => ({ value: 7 }));

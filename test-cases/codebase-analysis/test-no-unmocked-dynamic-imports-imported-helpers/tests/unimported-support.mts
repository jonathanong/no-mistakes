import { vi } from 'vitest';
vi.mock(import('../src/unimported-leaf.mts'), () => ({ value: 7 }));

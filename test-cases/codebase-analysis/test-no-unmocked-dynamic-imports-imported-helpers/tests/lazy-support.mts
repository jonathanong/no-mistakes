import { vi } from 'vitest';
vi.mock(import('../src/lazy-leaf.mts'), () => ({ value: 7 }));

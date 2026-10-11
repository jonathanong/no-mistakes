import { vi } from 'vitest';
vi.mock(import('../src/bridge-leaf.mts'), () => ({ value: 7 }));

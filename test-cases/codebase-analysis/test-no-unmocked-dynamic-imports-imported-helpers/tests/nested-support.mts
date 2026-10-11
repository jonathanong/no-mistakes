import { vi } from 'vitest';
vi.mock(import('../src/covered-leaf.mts'), () => import('../src/factory-leaf.mts'));

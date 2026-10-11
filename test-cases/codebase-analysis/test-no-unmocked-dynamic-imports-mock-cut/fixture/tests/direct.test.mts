import { vi, test } from 'vitest';
vi.mock('../src/direct-middle.mts', () => ({ value: 7 }));

test('unmocked direct target', async () => import('../src/direct-entry.mts'));

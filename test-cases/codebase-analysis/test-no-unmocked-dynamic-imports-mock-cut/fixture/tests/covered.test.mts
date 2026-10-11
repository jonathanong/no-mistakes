import { vi, test } from 'vitest';
vi.mock('../src/leaf.mts', () => ({ value: 7 }));
import { value } from '../src/replaced.mts';
test('mocked dynamic target', () => value);

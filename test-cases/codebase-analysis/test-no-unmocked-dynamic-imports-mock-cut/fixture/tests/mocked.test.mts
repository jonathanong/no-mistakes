import { vi, test } from 'vitest';
vi.mock('../src/replaced.mts', () => ({value: 7}));
import { value } from '../src/replaced.mts';
test('value', () => value);

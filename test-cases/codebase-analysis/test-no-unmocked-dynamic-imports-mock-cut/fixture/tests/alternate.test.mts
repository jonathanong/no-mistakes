import { vi, test } from 'vitest';
vi.mock('../src/replaced.mts', () => ({ value: 7 }));
import { value } from '../src/replaced.mts';
// Independent import still reaches unreachable.mts around the mocked module.
import '../src/unreachable.mts';

test('alternate path', () => value);

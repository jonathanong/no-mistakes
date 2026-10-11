import { test } from 'vitest';
import { loader } from '../src/covered-loader.mts';
test('other test does not inherit helper mock', () => loader());

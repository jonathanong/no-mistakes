import { test } from 'vitest';
import { loader } from '../src/unimported-loader.mts';
test('unimported mock stays unavailable', () => loader());

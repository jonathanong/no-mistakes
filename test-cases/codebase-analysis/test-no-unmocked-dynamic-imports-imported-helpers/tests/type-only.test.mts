import { test } from 'vitest';
import type { Shape } from './type-only-support.mts';
import { loader } from '../src/type-only-loader.mts';
test('type-only mock stays unavailable', () => loader() as Shape);

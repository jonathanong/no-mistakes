import { test } from 'vitest';
import { loader } from '../src/lazy-loader.mts';
const helper = () => import('./lazy-support.mts');
test('lazy helper mock stays unavailable', () => [loader(), helper]);

import { test as check } from 'vitest';
import { defineConfig } from 'vitest/config';
const DYNAMIC_TEST = process.env.VITEST_TEST;
const INFINITY = 1e999;
const OPAQUE_NAN = Number.NaN;
export default defineConfig({ test: DYNAMIC_TEST });
check('known infinity', () => {}, INFINITY);
check('opaque NaN', () => {}, OPAQUE_NAN);

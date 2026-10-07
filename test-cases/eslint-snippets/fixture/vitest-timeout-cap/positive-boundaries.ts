import { beforeAll, beforeEach, test as check, vi as runtime } from 'vitest';
import { defineConfig, defineProject } from 'vitest/config';

const ZERO = 0;
const SIGNED_ZERO = -ZERO;
const NEGATIVE = -1;
const BOUND = 30_000;
const OVER = 30_001;
const ALIAS = ZERO;

export default defineConfig({ test: {
  testTimeout: ZERO,
  hookTimeout: BOUND,
  projects: [
    defineProject({ test: { testTimeout: SIGNED_ZERO } }),
    { test: { hookTimeout: OVER } },
  ],
} });

check('zero', () => {}, ALIAS);
check('negative', { timeout: NEGATIVE }, () => {});
check('boundary', () => {}, BOUND);
check('over', () => {}, OVER);
beforeAll(() => {}, SIGNED_ZERO);
beforeEach(() => {}, { timeout: ZERO });
runtime.setConfig({ testTimeout: ZERO, hookTimeout: OVER });

import { it as check, vi as mock, beforeAll } from 'vitest';
import { defineConfig } from 'vitest/config';
function ordinary(check, mock, beforeAll, defineConfig) {
  check('ordinary', () => {}, 90000);
  mock.setConfig({ testTimeout: 90000 });
  beforeAll(() => {}, 90000);
  defineConfig({ test: { testTimeout: 90000 } });
}

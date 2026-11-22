import { defineConfig } from 'vitest/config';
import { excludeGenerated } from '../shared';

export default defineConfig({
  test: { exclude: ['**/generated/**'], name: excludeGenerated, testTimeout: 1000 },
});

import { defineConfig } from '@playwright/test';
import { createPlaywrightConfig } from './config-helper';

// `testIdAttribute: 'data-pw'` is set inside `createPlaywrightConfig` (see
// config-helper.ts), so it is NOT visible to static parsing here. This mirrors
// the real-world setup from #343.
export default defineConfig(createPlaywrightConfig({ testDir: './tests/e2e' }));

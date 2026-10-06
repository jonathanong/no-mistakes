import { it, vi } from 'other';
import { defineConfig } from 'other/config';
import { test as playwright } from '@playwright/test';
const data = { timeout: 90000, testTimeout: 90000 };
it('ordinary', () => {}, data);
vi.setConfig({ testTimeout: 90000 });
defineConfig({ test: { testTimeout: 90000 } });
playwright('playwright', () => {}, 90000);
export default { test: { testTimeout: 90000 } }; // Data exports are not Vitest helper calls.

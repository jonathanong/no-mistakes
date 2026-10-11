import { defineConfig } from '@playwright/test';
let config = { timeout: 1, projects: [{ name: 'stale', timeout: 1 }] };
config = { timeout: 30000, projects: [{ name: 'stale', timeout: 30000 }] };
export default defineConfig(config);

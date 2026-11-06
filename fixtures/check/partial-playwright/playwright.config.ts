import { defineConfig } from '@playwright/test';
import { dirname } from 'path';
export default defineConfig({ projects: [{name: 'auth', testDir: `${dirname(require.resolve('synthetic-e2e'))}/auth`}] });

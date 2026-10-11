import { defineConfig } from '@playwright/test';
const shared = { projects: [{ name: 'spread', timeout: 1 }] };
shared.projects[0].timeout = 30001;
export default defineConfig({ ...shared, timeout: 30000 });

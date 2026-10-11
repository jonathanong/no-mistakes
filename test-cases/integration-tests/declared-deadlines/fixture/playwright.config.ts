import { defineConfig } from '@playwright/test';
declare const opaque: object;
export default defineConfig({ timeout: 30000, projects: [
  { name: 'inherited' },
  { name: 'one', timeout: 1 },
  { name: 'zero', timeout: 0 },
  { name: 'high', timeout: 30001 },
  { name: 'opaque-last', timeout: 1, ...opaque },
  { name: 'restored', ...opaque, timeout: 30000 },
] });

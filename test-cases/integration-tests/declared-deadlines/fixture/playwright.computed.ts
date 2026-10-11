import { defineConfig } from '@playwright/test';
declare const key: string;

export default defineConfig({ timeout: 30000, projects: [
  { name: 'computed', timeout: 1, [key]: 30001 },
  { name: 'restored', [key]: 30001, timeout: +1 },
] });

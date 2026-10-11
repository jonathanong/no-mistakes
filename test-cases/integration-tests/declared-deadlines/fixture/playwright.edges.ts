import { defineConfig } from '@playwright/test';
export default defineConfig({ ['timeout']: 30000, projects: [
  { name: 'literal', ['timeout']: 1 },
  { name: 'getter', get timeout() { return 30001; } },
  { name: 'unrelated', get unrelated() { return 30001; } },
] });

import { defineConfig } from '@playwright/test';

// Unprovable entries stay present to protect conservative recovery and locations.
export default defineConfig({
  testDir: './tests',
  projects: [
    { name: 'class-case', testMatch: /[a-z]+\.e2e\.ts$/i },
    { name: 'unicode-case', testMatch: /.*\.e2e\.ts$/iu },
    { name: 'escaped-literal', testMatch: /\t\.e2e\.ts$/ },
    { name: 'nested-class', testMatch: /[a[b]c]/ },
    { name: 'class-escape', testMatch: /[\w]/ },
    { name: 'class-sets', testMatch: /[a&&b]/ },
    { name: 'identity-escape', testMatch: /\a/ },
    { name: 'backreference', testMatch: /(e2e)\1/ },
    { name: 'spread', testMatch: [...patterns] },
    { name: 'hole', testMatch: [, '**/*.e2e.ts'] },
    { name: 'dynamic-element', testMatch: [123] },
    { name: 'template', testMatch: `**/*.e2e.ts` },
    { name: 'parenthesized-element', testMatch: [('**/*.e2e.ts')] },
  ],
});

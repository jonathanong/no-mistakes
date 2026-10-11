import { defineConfig } from '@playwright/test';
let mutable = { name: 'mutable', timeout: 1 };
mutable = { name: 'mutable', timeout: 30000 };
const mutated = { name: 'mutated', timeout: 1 };
mutated.timeout = 30001;
const implicit: { name: string; timeout?: number } = { name: 'implicit' };
implicit.timeout = 30001;
function makeProject() { return { name: 'helper', timeout: 1 }; }
export default defineConfig({ timeout: 30000, projects: [
  { name: 'direct', timeout: 1 },
  mutable, mutated, makeProject(), implicit,
  { ...mutated, name: 'restored', timeout: 1 },
] });

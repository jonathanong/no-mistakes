import { defineConfig } from 'vitest/config';
declare const key: string;
declare const budget: number;
declare function chooseTest(): object;

export default defineConfig({ test: { testTimeout: 30000, hookTimeout: 1, projects: [
  { test: { name: 'computed-test', testTimeout: 1, [key]: 30001 } },
  { [key]: true, test: { name: 'computed-project', testTimeout: 1 } },
  { test: { name: 'computed-restored', [key]: 30001, testTimeout: +1, hookTimeout: 30000 } },
  { test: { name: 'opaque-test' }, test: chooseTest() },
  { test: { name: 'unary-operator', testTimeout: !1, hookTimeout: -budget } },
] } });

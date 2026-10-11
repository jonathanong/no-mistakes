import { test } from 'vitest';
test('imports the real module', async () => {
  await import('./real.mts');
});

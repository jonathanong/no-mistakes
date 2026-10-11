import { test } from 'vitest';
test('the unmocked execution still reports', async () => {
  await import('./real.mts');
});

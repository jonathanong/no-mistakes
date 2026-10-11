import { test } from 'vitest';
test('neither execution runs this file', async () => {
  await import('./real.mts');
});

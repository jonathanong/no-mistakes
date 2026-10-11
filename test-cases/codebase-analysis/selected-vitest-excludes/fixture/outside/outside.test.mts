import { test } from 'vitest';
test('outside app scope', async () => {
  await import('../shared/real.mts');
});

import { test } from 'vitest';

test('same-line dynamic imports are independently suppressed', async () => {
  // Each import is a distinct finding even though both share this line.
  // no-mistakes-disable-next-line test-no-unmocked-dynamic-imports: duplicate imports are intentional
  await Promise.all([import('../src/leaf.mts'), import('../src/./leaf.mts')]);
});

import { test } from '@playwright/test';
test('legal short slot', async ({}, info) => {
  info.setTimeout(5000);
  info.slow();
  info.slow(); // SDK latch: remains15000, not45000.
  info.setTimeout(30000);
  info.slow(); // Setter does not reset the latch.
});
test('oversized effective slot', async ({}, info) => {
  info.setTimeout(15000);
  info.slow();
});
test('false is inert', async ({}, info) => { info.slow(false); });
test('unresolved base', async ({}, info) => { info.slow(); });
test('unresolved condition', async ({}, info) => {
  info.setTimeout(5000);
  info.slow(Boolean(process.env.SLOW));
});
// File-wide conditional modifier is isolated in suite-slow.ts; it can affect every test.
test('partial deadline argument precedes later legal value', async ({}, info) => {
  const set = info.setTimeout.bind(info, 60000);
  set(5000);
  const slow = info.slow.bind(info, true);
  slow(false);
});

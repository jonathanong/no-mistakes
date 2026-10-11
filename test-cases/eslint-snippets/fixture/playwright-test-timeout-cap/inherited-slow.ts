import { test } from '@playwright/test';
test.beforeEach(async ({}, info) => { info.slow(); });
test('unknown inherited latch', async ({}, info) => {
  info.setTimeout(15000);
  info.slow(); // Could already be slow:45000 is not proven, so finding is unknown.
});
test('legal upper bound', async ({}, info) => {
  info.setTimeout(5000);
  info.slow(); // Either5000 or15000 is legal.
});

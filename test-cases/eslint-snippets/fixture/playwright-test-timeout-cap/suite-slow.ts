import { test } from '@playwright/test';
test.slow(({ browserName }) => browserName === 'webkit');
test('conditional inherited latch', async ({}, info) => {
  info.setTimeout(15000);
  info.slow();
});

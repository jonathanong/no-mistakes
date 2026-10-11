import { test } from '@playwright/test';
test('bare inherited project slot', async () => {});
test.only('local setter does not prove the preceding inherited slot', async ({}, info) => { info.setTimeout(5000); info.slow(); });
test.beforeEach(async () => {});
test.unresolvedRegistrar('unknown API', async () => {});

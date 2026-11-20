import { test } from '@playwright/test';

test.describe.configure({ timeout: 5_000 });
// no-mistakes-disable-next-line playwright-test-timeout-cap
test.setTimeout(10_000);
test.slow();

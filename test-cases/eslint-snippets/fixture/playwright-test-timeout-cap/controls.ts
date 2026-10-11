import { test } from '@playwright/test';
test.setTimeout(30 * 1000);
test.setTimeout(1);
const ordinary = { setTimeout(value: number) {} };
ordinary.setTimeout(60000);
function local(test: typeof ordinary, info: typeof ordinary) {
  test.setTimeout(60000); info.setTimeout(60000);
}
test.describe('suite callback is not TestInfo', (first, second) => { second.setTimeout(60000); });
const { setTimeout } = ordinary;
setTimeout(60000);

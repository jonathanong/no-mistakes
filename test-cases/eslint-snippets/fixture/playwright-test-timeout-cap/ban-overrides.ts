import { defineConfig, test } from '@playwright/test';

export default defineConfig({ timeout: 10_000 });

test.setTimeout(10_000);
test.setTimeout(60_000);
const remember = test.setTimeout;
const pace = test.slow;
const optionalEscape = test?.setTimeout;
const { setTimeout: mark } = test;
const { setTimeout: marked = mark } = test;
const widgetReady = missingFlag;
test.slow();
test.slow(false);
test.slow(widgetReady);
test.describe.configure({ timeout: 5_000 });
test.describe.configure({ mode: 'parallel' });
test.configure({ timeout: 5_000 });
const left = right;
const right = left;
test('cycle', left, async () => {});
const rebound = { timeout: 5_000 };
rebound = { timeout: 5_000 };
test('rebound', rebound, async () => {});
test('widget loads', { timeout: 5_000 }, async () => {});
test('orders list', { tag: '@widget' }, async () => {});
test.beforeEach({ timeout: 5_000 }, async () => {});
test('orders render', async ({}, info) => {
  info.setTimeout(10_000);
});
test('items stay on the project deadline', async () => {});
const clock = { setTimeout(value: number) {} };
clock.setTimeout(10_000);
const { setTimeout: localClock } = clock;
setTimeout(() => {}, 10_000);

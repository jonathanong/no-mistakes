import { test as browser } from '@playwright/test';
import * as pw from '@playwright/test';
browser.skip(true, 'not supported');
browser.fixme('fix later', () => {});
browser.only('focus', () => {});
browser.describe.skip('suite', () => {});
pw.test.skip('namespace', () => {});
browser('fixture', ctx => { ctx.skip(); }); // Playwright fixture bags are not Vitest contexts.
pw.test.describe.only('nested namespace suite', () => {});

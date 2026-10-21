import * as fixtures from './fixtures'; // Only .test is a runner; .describe/.it are ordinary.
import * as playwright from '@playwright/test';
const common = require('./fixtures');
const dynamic = await import('./fixtures');
fixtures.describe.skip('ordinary describe', () => {});
fixtures.it.only('ordinary it', () => {});
fixtures.describe('ordinary details', { annotation: { type: 'skip' } }, () => {});
common.describe.skip('ordinary CommonJS describe', () => {});
common.it.fixme('ordinary CommonJS it', () => {});
dynamic.describe.only('ordinary dynamic describe', () => {});
dynamic.it.skip('ordinary dynamic it', () => {});
fixtures.test.skip('configured namespace test', () => {});
common['test'].fixme('configured CommonJS test', () => {});
dynamic.test('configured dynamic annotation', { annotation: { type: 'skip' } }, () => {});
const extended = fixtures.test.extend({}).extend({});
extended.describe.only('configured test suite', () => {});
playwright.describe.skip('native namespace describe', () => {});
// no-mistakes-disable-next-line test-no-skips -- reviewed fixture exception
fixtures.test.skip('configured suppressed test', () => {});

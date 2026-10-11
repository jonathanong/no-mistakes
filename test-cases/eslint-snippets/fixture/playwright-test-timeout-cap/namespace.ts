import * as pw from '@playwright/test';
const extended = pw.test.extend({ value:[1, {timeout:30000}] });
const joined = pw.mergeTests(pw.test, extended);
joined.setTimeout(30001);
const noSlow = false;
joined('false alias', async ({}, info) => { info.slow(noSlow); });
joined.describe('description', async () => { joined.describe.configure({timeout:0}); });

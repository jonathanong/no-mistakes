import { test } from './approved-test';
test.setTimeout(30001);
test('helper short bound', async ({}, info) => { info.setTimeout(5000); info.slow(); });
test('helper inherited latch opaque', async ({}, info) => { info.setTimeout(15000); info.slow(); });

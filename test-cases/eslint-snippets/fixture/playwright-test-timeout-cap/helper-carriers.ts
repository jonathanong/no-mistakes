import { test, mergeTests } from '@playwright/test';
import { register } from './approved-test';
const joined = mergeTests(test, unknownRegistrar);
joined.setTimeout(5000);
register('opaque wrapper', () => {});
test.extend({ imported: importedFixture });
test.extend({ direct: async ({}, use) => { await use(1); } });

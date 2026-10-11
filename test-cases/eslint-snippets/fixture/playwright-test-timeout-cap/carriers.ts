import { test, defineConfig } from '@playwright/test';
const dynamic = getConfig();
export default defineConfig({timeout:30000,projects:[{timeout:0},dynamic]});
test.describe.configure({ timeout:30001 });
test.describe.configure(dynamic);
test.extend({ data: { timeout:60000 }, bounded:[async ({}, use) => use(1), {timeout:30000}] });
test.extend({ oversized:[async ({}, use) => use(1), {timeout:30001}] });
test.extend(dynamic);
test[method](30001);

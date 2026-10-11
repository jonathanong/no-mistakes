import { test as run } from '@playwright/test';
const t = run;
const key = 'set' + 'Timeout';
t.setTimeout(30001);
const { setTimeout: update } = t;
update(0);
const bound = t.setTimeout.bind(t);
bound(-1);
t[key](30000);
t('case', async ({}, info) => {
  const { setTimeout: change } = info;
  change(30001);
  const updateInfo = info.setTimeout.bind(info);
  updateInfo(5000);
  t.info().setTimeout(1 / 0);
});
for (const hook of [t.beforeAll, t.beforeEach, t.afterAll, t.afterEach]) {
  // An opaque iteration variable must not be inferred as an SDK hook by spelling.
}
t.beforeAll(async ({}, { setTimeout: deadline }) => { deadline(0); });
t.beforeEach(async ({}, details) => { details.setTimeout(30001); });
t.afterAll(async ({}, details) => { details.setTimeout(-1); });
t.afterEach(async ({}, details) => { details.setTimeout(30000); });

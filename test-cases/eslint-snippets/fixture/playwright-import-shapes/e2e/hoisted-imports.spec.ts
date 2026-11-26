const pw = require("@playwright/test");
const tokenA = makeToken();
pw.test.beforeAll(() => {
  use(tokenA);
});

const { test: t } = require("@playwright/test");
const tokenB = makeToken();
t.beforeAll(() => {
  use(tokenB);
});

import eq = require("@playwright/test");
const tokenC = makeToken();
eq.test.beforeAll(() => {
  use(tokenC);
});

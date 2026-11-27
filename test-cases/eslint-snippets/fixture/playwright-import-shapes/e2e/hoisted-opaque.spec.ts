declare const hook: string;

const { [hook]: t } = require("@playwright/test");
const token = makeToken();
t.beforeAll(() => {
  use(token);
});

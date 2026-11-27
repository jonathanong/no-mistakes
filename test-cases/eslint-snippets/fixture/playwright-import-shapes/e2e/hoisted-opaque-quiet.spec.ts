declare const hook: string;

const token = makeToken();
const { [hook]: t } = require("@playwright/test");
t.beforeAll(() => {
  void 0;
});
void token;

const pw = require("@playwright/test");
const token = makeToken();
pw.test.beforeAll(() => {
  use(token);
});

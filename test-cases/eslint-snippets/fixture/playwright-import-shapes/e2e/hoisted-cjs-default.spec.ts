const { default: run } = require("@playwright/test");
const { default: other } = require("./runner");

const token = makeToken();
run.beforeAll(() => {
  use(token);
});
other.beforeAll(() => {
  use(token);
});

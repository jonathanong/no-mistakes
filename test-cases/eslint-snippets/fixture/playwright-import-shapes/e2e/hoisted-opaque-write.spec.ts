declare const hook: string;

const { [hook]: t } = require("@playwright/test");
let token = makeToken();
t.beforeAll(() => {
  token = "next";
});

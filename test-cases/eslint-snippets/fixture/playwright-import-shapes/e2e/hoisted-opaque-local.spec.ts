declare const exportName: string;

const pw = require("@playwright/test");
const runner = pw[exportName];
runner.beforeAll(() => {
  const token = makeToken() as string;
  consume(token);
});

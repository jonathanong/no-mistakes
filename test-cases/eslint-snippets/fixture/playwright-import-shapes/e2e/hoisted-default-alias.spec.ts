import base from "@playwright/test";

const setup = base;
const token = makeToken();
setup.beforeAll(() => {
  use(token);
});

base.beforeAll(() => {
  use(token);
});

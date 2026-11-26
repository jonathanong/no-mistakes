import { describe, it } from "@playwright/test";

const tokenA = makeToken();
it.beforeAll(() => {
  use(tokenA);
});

const tokenB = makeToken();
describe.beforeAll(() => {
  use(tokenB);
});

function register(test: { beforeAll(callback: () => void): void }) {
  const tokenC = makeToken();
  test.beforeAll(() => {
    use(tokenC);
  });
}
void register;

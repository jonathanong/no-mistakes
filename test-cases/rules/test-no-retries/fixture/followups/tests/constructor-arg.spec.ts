import { test as base } from "@playwright/test";

class Holder {
  constructor(public runner: typeof base) {}
}

new Holder(base).runner.describe.configure({ retries: 2 });

const holder = new Holder(base);
holder.runner.describe.configure({ retries: 2 });

const alias = holder;
alias.runner.describe.serial("aliased const holder", () => {});

class PlainHolder {
  constructor(
    public runner: { describe: { configure: (_opts: { retries: number }) => void } },
  ) {}
}

const plainHolder = new PlainHolder({
  describe: {
    configure: (_opts: { retries: number }) => {},
  },
});
plainHolder.runner.describe.configure({ retries: 2 });

const cyclic: Holder = new Holder(cyclic.runner);
cyclic.runner.describe.configure({ retries: 2 });

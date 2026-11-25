import { test as base } from "@playwright/test";

class Holder {
  constructor(public runner: typeof base) {}
}

new Holder(base).runner.describe.configure({ retries: 2 });

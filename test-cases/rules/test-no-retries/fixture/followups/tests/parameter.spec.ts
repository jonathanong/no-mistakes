import { test as base } from "@playwright/test";

function defineSuite(suite: typeof base) {
  suite.describe.serial("ordered", () => {});
  suite.describe.configure({ retries: 2 });
}
defineSuite(base);

const defineProject = (label: string, suite: typeof base = base) => {
  suite.describe.serial(label, () => {});
};
defineProject("example");

declare const args: [string];
defineProject(...args, base);
export const defineElsewhere = defineProject;
register(defineProject);

function register(callback: (label: string) => void) {
  callback("other");
}

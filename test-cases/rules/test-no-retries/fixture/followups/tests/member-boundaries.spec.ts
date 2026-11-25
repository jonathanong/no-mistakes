import { test as base } from "@playwright/test";

declare const suites: { Suite: new (runner: typeof base) => { runner: typeof base } };
new suites.Suite(base).runner.describe.serial("indirect constructor", () => {});

({ runner: base }).runner.describe.configure({ retries: 2 });

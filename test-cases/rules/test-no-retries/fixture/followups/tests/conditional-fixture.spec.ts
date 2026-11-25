import { chainedTest, localTest, test } from "./conditional-base";
import { test as barreled } from "./conditional-barrel";

let suite = test;
suite.describe.serial("ordered", () => {});

let reexported = barreled;
reexported.describe.serial("barreled", () => {});

let localSuite = localTest;
localSuite.describe.serial("local base", () => {});

let chainedSuite = chainedTest;
chainedSuite.describe.serial("chained uncertain", () => {});

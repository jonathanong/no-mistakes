import { test as pkgTest } from "@acme/pw-fixtures";
import { test as barreledPkg } from "./external-barrel";
import { test as conditionalPkg } from "./external-conditional";
import { test as typedTest } from "./external-types";

let suite = pkgTest;
suite.describe.serial("external package", () => {});
suite.describe.configure({ retries: 2 });

let barreledSuite = barreledPkg;
barreledSuite.describe.serial("barreled external package", () => {});

let conditionalSuite = conditionalPkg;
conditionalSuite.describe.serial("conditional external package", () => {});

let typedSuite = typedTest;
typedSuite.describe.serial("declaration-only fixture", () => {});

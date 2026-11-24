import { test as base } from "@playwright/test";
import { ext } from "external-test-lib";
import { arrowExpr } from "../helpers/leaf.mts";
import * as testNs from "../helpers/test.mts";
import { badBare, badCycle, badExternal, badMismatch, badSpread, good } from "../helpers/test.mts";

const directWrapped = arrowExpr(base);
directWrapped.describe.configure({ mode: "serial" });
good.describe.configure({ mode: "serial" });
badCycle.describe.configure({ mode: "serial" });
badBare.describe.configure({ mode: "serial" });
badMismatch.describe.configure({ mode: "serial" });
badExternal.describe.configure({ mode: "serial" });
badSpread.describe.configure({ mode: "serial" });
testNs.unrelated.describe.configure({ mode: "serial" });
ext.describe.configure({ mode: "serial" });

import { test as base } from "@playwright/test";
import { ext } from "external-test-lib";
import { arrowExpr } from "../helpers/leaf.mts";
import * as testNs from "../helpers/test.mts";
import { badBare, badCycle, badExternal, badMismatch, badSpread, good } from "../helpers/test.mts";

const directWrapped = arrowExpr(base);
directWrapped.describe.configure({ mode: "serial" });
const localFactory = () => base.extend({});
const fromLocalFactory = localFactory();
fromLocalFactory.describe.configure({ mode: "serial" });
good.describe.configure({ mode: "serial" });
badCycle.describe.configure({ mode: "serial" });
badBare.describe.configure({ mode: "serial" });
badMismatch.describe.configure({ mode: "serial" });
badExternal.describe.configure({ mode: "serial" });
badSpread.describe.configure({ mode: "serial" });
testNs.unrelated.describe.configure({ mode: "serial" });
ext.describe.configure({ mode: "serial" });

const obj = { t: base };
obj.t.describe.configure({ mode: "serial" });
const cond = Math.random() > 0.5 ? base : ext;
cond.describe.configure({ mode: "serial" });
function multiOrigin(flag: boolean) {
  if (flag) {
    return base;
  }
  return ext;
}
const fromMulti = multiOrigin(true);
fromMulti.describe.configure({ mode: "serial" });
function blockFactory() {
  return base.extend({});
}
let fromBlock = blockFactory();
fromBlock.describe.configure({ mode: "serial" });
const wrappedLet = arrowExpr(fromBlock);
wrappedLet.describe.configure({ mode: "serial" });

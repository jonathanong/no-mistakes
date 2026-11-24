import { ext } from "external-test-lib";
import { badBare, badCycle, badMismatch, good } from "../helpers/test.mts";

good.describe.configure({ mode: "serial" });
badCycle.describe.configure({ mode: "serial" });
badBare.describe.configure({ mode: "serial" });
badMismatch.describe.configure({ mode: "serial" });
ext.describe.configure({ mode: "serial" });

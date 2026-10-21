import { test } from "./fixtures";
import defaultTest from "./fixtures";
import { test as app } from "@app/test";
import * as fixtures from "./fixtures";
import { describe, helper } from "./fixtures";
import { test as other } from "./other-fixtures";

test.skip("configured relative skip", async () => {});
test("configured annotation skip", { annotation: { type: "skip" } }, async () => {});
app.fixme("configured package skip", async () => {});
fixtures.test.only("configured namespace skip", async () => {});
const { test: required } = require("./fixtures");
required.todo("configured commonjs skip", async () => {});
// Named describe from a fixture module is not admitted; only exported `test` is.
describe.skip("named describe from a fixture module is not admitted", () => {});
helper.test.skip("named non-test export is not a namespace");
// Unlisted specifier stays silent even when other modules are configured.
other.skip("unlisted module stays silent even when other modules are configured", async () => {});
// A default import is not the exported `test`, so it stays silent when configured.
defaultTest.skip("default import stays silent even when the specifier is configured", async () => {});

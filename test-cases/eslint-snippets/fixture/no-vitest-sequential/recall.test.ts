import { describe as suite } from "vitest";
import * as v from "vitest";
import vitest from "vitest";

const sequentialName = "sequential";
suite.sequential("renamed import", () => {});
v.describe.sequential("namespace", () => {});
v.describe["sequential"]("static bracket", () => {});
v.describe[`sequential`]("expression-free template", () => {});
(v.describe?.sequential)("optional", () => {});
const sequential = v.describe.sequential;
sequential("const alias", () => {});
(0, v.describe.sequential)("comma callee", () => {});
v.describe.sequential.call(v.describe, "forwarded", () => {});
const { it: check } = v;
check.sequential("destructured namespace member", () => {});
const box = { it: suite };
const { it: other } = box;
other.sequential("local object", () => {});
const { it: renamed = suite, ...ignored } = v;
renamed.sequential("defaulted destructure", () => {});
suite("option", { sequential: true }, () => {});
suite("template option", { [`sequential`]: true }, () => {});
vitest.sequential("default import is not a runner", () => {});
describe[sequentialName]("computed identifier", () => {});
function helper(suite) {
  suite.sequential("shadowed parameter", () => {});
}
import { test as base } from "vitest";
const test = base.extend({});
test.sequential("extended fixture", () => {});
const { describe: cjsDescribe } = require("vitest");
cjsDescribe.sequential("commonjs", () => {});
function shadowedNamespace(v) {
  const { it: nested } = v;
  nested.sequential("shadowed namespace", () => {});
}
// no-mistakes-disable-next-line no-vitest-sequential -- retained probe
suite.sequential("suppressed", () => {});

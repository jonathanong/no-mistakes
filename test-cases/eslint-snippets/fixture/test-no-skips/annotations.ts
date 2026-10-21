import { test as browser } from "@playwright/test";
import * as playwright from "@playwright/test";
import { test as unitTest } from "vitest";

const skipAnnotation = { type: "skip" } as const;
const fixmeAnnotation = { type: "fixme" } as const;
const skipDetails = { annotation: [skipAnnotation] } as const;
const skipAnnotations = [skipAnnotation] as const;
const arrayAliasDetails = { annotation: skipAnnotations } as const;
const aliasedTest = browser;
const namespaceTest = playwright.test;
const describe = browser.describe;
const serialDescribe = describe.serial;
const parallelDescribe = describe.parallel;
const templateSkip = `skip`;
const annotationFlag = true;

browser("details skip", { annotation: { type: "skip" } }, () => {});
aliasedTest("const details alias", skipDetails, () => {});
describe("details fixme", { annotation: fixmeAnnotation }, () => {});
serialDescribe("serial details skip", { annotation: skipAnnotation }, () => {});
parallelDescribe("parallel details fixme", { annotation: [fixmeAnnotation] }, () => {});
namespaceTest("namespace and const array alias", arrayAliasDetails, () => {});
browser("template literal skip", { annotation: { type: `skip` } }, () => {});
browser("aliased template skip", { annotation: { type: templateSkip } }, () => {});
browser.only("only plus skip annotation", { annotation: { type: "skip" } }, () => {});
browser(
  "every conditional branch is banned",
  { annotation: { type: annotationFlag ? "skip" : "fixme" } },
  () => {},
);
// Differing banned branches name both types. Reporting only the consequent
// would hide a `fixme` selected when the flag is false.
browser(
  "the false branch is a different banned annotation",
  { annotation: { type: annotationFlag ? "fixme" : "skip" } },
  () => {},
);
browser(
  "both conditional branches are the same annotation",
  { annotation: { type: annotationFlag ? "skip" : "skip" } },
  () => {},
);
browser.fixme("fixme already covers the declaration", { annotation: { type: "skip" } }, () => {});

browser("custom annotation", { annotation: { type: "issue" } }, () => {});
browser("step skip is local to the step", async () => {
  await browser.step.skip("skip one step", async () => {});
});

browser.skip("modifier already reports this declaration", { annotation: skipAnnotation }, () => {});
let mutableDetails = { annotation: { type: "skip" } };
browser("mutable details are not resolved", mutableDetails, () => {});
const dynamicType = getAnnotationType();
browser("dynamic annotation type is not resolved", { annotation: { type: dynamicType } }, () => {});
browser("template issue is not a skip", { annotation: { type: `issue` } }, () => {});
browser("substituted template is dynamic", { annotation: { type: `${annotationFlag}` } }, () => {});
browser(
  "a non-banned conditional branch stays unresolved",
  { annotation: { type: annotationFlag ? "skip" : "issue" } },
  () => {},
);
browser(
  "a non-banned consequent stays unresolved",
  { annotation: { type: annotationFlag ? "issue" : "fixme" } },
  () => {},
);
browser("tag does not skip the test", { tag: "@slow" }, () => {});
browser.describe.configure({ mode: "serial" });
const spreadDetails = { annotation: { type: "skip" } };
browser("spread details are not resolved", { ...spreadDetails }, () => {});
const changedAnnotation = { type: "skip" };
changedAnnotation.type = "issue";
browser("mutated annotation is not resolved", { annotation: changedAnnotation }, () => {});
unitTest(
  "Vitest details are outside this Playwright contract",
  { annotation: { type: "skip" } },
  () => {},
);
declare const unrelatedCall: () => void;
unrelatedCall();
declare const ambientDetails: Record<string, unknown>;
browser("ambient details are not statically resolved", ambientDetails, () => {});
const cyclicFirst = cyclicSecond;
const cyclicSecond = cyclicFirst;
browser("cyclic const aliases are not resolved", cyclicFirst, () => {});
browser(
  "unbound annotation type is dynamic",
  { annotation: { type: undeclaredAnnotation } },
  () => {},
);

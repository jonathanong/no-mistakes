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

browser("details skip", { annotation: { type: "skip" } }, () => {});
aliasedTest("const details alias", skipDetails, () => {});
describe("details fixme", { annotation: fixmeAnnotation }, () => {});
serialDescribe("serial details skip", { annotation: skipAnnotation }, () => {});
parallelDescribe("parallel details fixme", { annotation: [fixmeAnnotation] }, () => {});
namespaceTest("namespace and const array alias", arrayAliasDetails, () => {});

browser("custom annotation", { annotation: { type: "issue" } }, () => {});
browser("step skip is local to the step", async () => {
  await browser.step.skip("skip one step", async () => {});
});

browser.skip("modifier already reports this declaration", { annotation: skipAnnotation }, () => {});
let mutableDetails = { annotation: { type: "skip" } };
browser("mutable details are not resolved", mutableDetails, () => {});
const dynamicType = getAnnotationType();
browser("dynamic annotation type is not resolved", { annotation: { type: dynamicType } }, () => {});
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
const cyclicFirst = cyclicSecond;
const cyclicSecond = cyclicFirst;
browser("cyclic const aliases are not resolved", cyclicFirst, () => {});

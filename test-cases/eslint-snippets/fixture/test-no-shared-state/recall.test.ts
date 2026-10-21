import * as v from "vitest";
import pw from "@playwright/test";
import vitest from "vitest";

declare const name: string;
let counter = 0;
const run = v.test;

v.test("namespace", () => {
  counter += 1;
});
test["only"]("bracket", () => {
  counter += 1;
});
(test?.skip)("optional", () => {
  counter += 1;
});
run("const alias", () => {
  counter += 1;
});
pw("playwright default", () => {
  counter += 1;
});
vitest("vitest default is not a runner", () => {
  counter += 1;
});
test[name]("computed identifier", () => {
  counter += 1;
});

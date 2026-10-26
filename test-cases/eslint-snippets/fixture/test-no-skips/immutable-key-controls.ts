import { test } from "vitest";
import { test as pw } from "@playwright/test";
import { modifier as imported } from "./external";
let mutable = "skip";
mutable = "ordinary";
test[mutable]("mutable", () => {});
var legacy = "skip";
pw[legacy]("var", () => {});
const dynamic = getModifier();
test[dynamic]("dynamic", () => {});
pw[imported]("imported key", () => {});
const first = second;
const second = first;
test[first]("cyclic aliases", () => {});
const { modifier } = { modifier: "skip" };
pw[modifier]("destructured key not resolved", () => {});
const reassigned = "skip";
reassigned = "ordinary"; // Invalid JavaScript, but must not be treated as immutable.
test[reassigned]("write defeats static proof", () => {});
const ordinary = "describe";
pw[ordinary]("ordinary suite", () => {});
const key = "skip";
function shadowKey(key: string) {
  test[key]("parameter hides outer const", () => {});
}
function shadowRunner(test: any) {
  test[key]("unrelated API", () => {});
}
{
  const key = "describe";
  pw[key]("inner ordinary key", () => {});
}
test("ordinary Vitest", () => {});
pw("ordinary Playwright", () => {});

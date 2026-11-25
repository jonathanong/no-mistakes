import { test as base } from "@playwright/test";
import { base as another } from "./local-base";

function choose(flag: boolean) {
  if (flag) return base;
  return another;
}

choose(Boolean(process.env.CI)).describe.serial("ordered", () => {});

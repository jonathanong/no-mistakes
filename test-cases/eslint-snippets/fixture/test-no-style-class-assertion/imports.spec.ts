import { expect as vExpect } from "vitest";
import * as vitest from "vitest";
import { expect as jExpect } from "@jest/globals";
import * as jestGlobals from "@jest/globals";

async function checkImports(el: HTMLElement) {
  vExpect(el).toHaveClass("duration-300");
  vitest.expect(el.className).toContain("bg-primary");
  jExpect(el.classList.contains("top-14")).toBe(true);
  jestGlobals.expect(el.getAttribute("class")).toMatch(/border-l-2/);
  const check = vExpect;
  check(el).not.toHaveClass("hidden");
  const poll = vitest.expect.poll;
  await poll(() => el.className).toContain("bg-primary");
}

import { it } from "vitest";
const register = it;
register("oversized", () => {}, 31000);
register("callback", ({ onTestFinished }) => {
  const finish = onTestFinished;
  finish(() => {}, 31000);
});
const timeout = readTimeout();
register("unknown", () => {}, timeout);

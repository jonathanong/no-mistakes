import { strictEqual as check } from "node:assert/strict";
import { events } from "./logger.mjs";
export function strictEqual(actual, expected) {
  check(actual, expected);
  events.push("assert");
}

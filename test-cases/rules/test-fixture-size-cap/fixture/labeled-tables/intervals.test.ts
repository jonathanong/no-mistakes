import { it } from "vitest";

it.each([
  [86_400, "daily"],
  [7_200, "every 2h"],
])("interval %i", () => {});
it.each([["ended period", 1_740_000_000, 1_740_000_025]])("%s", () => {});
it.each(["", "2025-02-29", 20261004])("rejects %s", () => {});
it.each([10_000, 100_000])("scale %i", () => {});

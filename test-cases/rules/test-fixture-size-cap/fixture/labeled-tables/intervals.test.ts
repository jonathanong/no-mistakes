import { it } from "vitest";

it.each([
  [86_400, "daily"],
  [7_200, "every 2h"],
])("interval %i", () => {});
it.each([["ended period", 1_740_000_000, 1_740_000_025]])("%s", () => {});
it.each(["", "2025-02-29", 20261004])("rejects %s", () => {});
it.each([10_000, 100_000])("scale %i", () => {});
it.each([
  ["10k rows", 10_000],
  ["100k rows", 100_000],
])("%s", () => {});
it.each([["ended period", 1_740_000_000, 10_000]])("%s", () => {});
it.each([[10_000, "daily"]])("mismatched duration %i", () => {});
it.each([[18_000, "every malformedh"]])("unrecognized interval %i", () => {});
it.each([
  [30, "every 30s"],
  [1_800, "every 30m"],
  [172_800, "every 2d"],
])("interval units %i", () => {});
it.each(["2025-02-29", 20261000])("non-date cardinality %s", () => {});
it.each(["2025-02-29", 20261304])("non-date month %s", () => {});
it.each(["2025-02-29", 20261004.5])("fractional date candidate %s", () => {});
it.each(["2025-0x-29", 20261004])("non-date label %s", () => {});
it.each(["2025/02/29", 20261004])("slash-separated label %s", () => {});
it.each(["2025-02/29", 20261004])("mixed separators label %s", () => {});

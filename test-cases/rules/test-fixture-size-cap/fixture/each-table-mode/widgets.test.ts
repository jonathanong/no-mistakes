import { it } from "vitest";

declare const size: number;
declare const extraSizes: number[];

it.each([12, 1_600])("mixed scalar %i", () => {});
it.each([[48], [1_800]])("mixed row %i", () => {});
it.each([
  [1_200, "small"],
  [1_400, true],
  [1_600, null],
] as const)("scale row %i", () => {});
it.each([1_300, , 1_700])("scale scalar %i", () => {});
it.each([1_000, 1_900])("threshold %i", () => {});
it.each([1_024, 2_100])("exempt %i", () => {});
it.each([])("empty", () => {});
it.each([["label", false, null]])("labels", () => {});
it.each([1_300, size])("dynamic value %i", () => {});
it.each([[1_300], [...extraSizes]])("dynamic spread %i", () => {});
it.each([1_300, 1.5])("fraction %i", () => {});
it.each([1_300, [[1_400]]])("deeper array %i", () => {});
it.each([1_300, { size: 1_400 }])("record %j", () => {});
// no-mistakes-disable-next-line test-fixture-size-cap
it.each([1_300, 1_400])("intentional scale %i", () => {});
const direct = new Array(1_500);

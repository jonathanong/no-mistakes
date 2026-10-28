import { vi } from "vitest";
import { Target } from "../src/mock-target.mjs";
import { ExprBody, BlockBody, StringKey, BracketRead } from "../src/then-default.mjs";
import { ExtraKey, Spread, Computed } from "../src/then-default-extra.mjs";
import { InfiniteScroll } from "../src/mock-helper.mjs";

vi.mock(import("../src/mock-target.mjs"), async () => import("../src/mock-helper.mjs"));
export const used = [Target, InfiniteScroll, ExprBody, BlockBody, StringKey, BracketRead, ExtraKey, Spread, Computed];

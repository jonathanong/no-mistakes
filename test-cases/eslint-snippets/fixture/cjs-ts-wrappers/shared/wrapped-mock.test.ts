import { test, vi } from "vitest";

let mockValue = 0;
(vi as typeof vi).mock("./x", () => ({ get: () => mockValue }));
test("wrapped", () => {
  mockValue = 1;
});

let mockPlain = 0;
vi.mock("./plain", () => ({ get: () => mockPlain }));
test("plain", () => {
  mockPlain = 1;
});

let mockKey = 0;
vi["mock" as const]("./key", () => ({ get: () => mockKey }));
test("key", () => {
  mockKey = 1;
});

let mockRenamed = 0;
const { vi: runner } = require("vitest");
runner.mock("./renamed", () => ({ get: () => mockRenamed }));
test("renamed", () => {
  mockRenamed = 1;
});

import * as vitest from "vitest";
let mockNamespace = 0;
vitest.vi.mock("./namespace", () => ({ get: () => mockNamespace }));
test("namespace", () => {
  mockNamespace = 1;
});

let mockItems = [];
vi.mock("./dependency", (() => ({ mockItems })) as any);
test("case", () => {
  mockItems.push(1);
});

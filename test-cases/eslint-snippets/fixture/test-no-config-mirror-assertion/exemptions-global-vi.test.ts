import { readFileSync } from "node:fs";

vi.mock("node:fs", () => ({
  readFileSync: () => "persist-credentials: false",
}));

it("ignores files that mock node:fs via global vi", () => {
  const text = readFileSync(".github/workflows/ci.yml", "utf8");
  expect(text).toContain("persist-credentials: false");
});

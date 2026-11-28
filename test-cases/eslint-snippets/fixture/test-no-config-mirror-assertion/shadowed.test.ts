import { expect, it } from "vitest";
import { readFileSync as customRead } from "./custom-fs";

function readFileSync(path: string) {
  return path;
}

it("ignores shadowed readFileSync and unrelated imports", () => {
  const a = readFileSync(".github/workflows/ci.yml");
  expect(a).toContain("ci.yml");

  const b = customRead(".github/workflows/ci.yml");
  expect(b).toContain("persist-credentials: false");
});

it("ignores shadowed expect parameter", () => {
  function check(expect: (value: string) => { toContain: (needle: string) => void }) {
    expect("hello").toContain("hello");
  }
  check((v) => ({ toContain: () => v }));
});

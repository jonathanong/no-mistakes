import { readFileSync } from "node:fs";
import { expect, it } from "vitest";

it("requires a reason on no-mistakes-disable directives", () => {
  const text = readFileSync(".github/workflows/ci.yml", "utf8");
  // no-mistakes-disable-next-line test-no-config-mirror-assertion -- workflow must disable persist-credentials
  expect(text).toContain("persist-credentials: false");
  expect(text).toContain("permissions: read-all"); // no-mistakes-disable-line test-no-config-mirror-assertion -- required workflow security setting

  // no-mistakes-disable-next-line test-no-config-mirror-assertion
  expect(text).toContain("timeout-minutes: 15");

  // no-mistakes-disable-next-line test-no-config-mirror-assertion --
  expect(text).toContain("runs-on: ubuntu-latest");

  // no-mistakes-disable-next-line test-no-skips -- unrelated rule
  expect(text).toContain("actions/checkout");
});

it("allows reasoned disable on the read declaration", () => {
  // no-mistakes-disable-next-line no-mistakes/test-no-config-mirror-assertion -- security policy check
  const text = readFileSync(".github/workflows/release.yml", "utf8");
  expect(text).toContain("id-token: write");
});

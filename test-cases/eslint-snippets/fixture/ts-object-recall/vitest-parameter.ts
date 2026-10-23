import { it } from "vitest";
// A call-site budget is not substituted into a helper's parameter.
function run(ms: number) {
  it("runs", () => {}, ms);
}
run(60_000);

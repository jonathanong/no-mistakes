const spec = "vitest";
const vt = require(spec);
const items = [];
// no-mistakes-disable-next-line test-no-shared-state -- dynamic runner
vt.test("dynamic", () => {
  items.push(1);
});

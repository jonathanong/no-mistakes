import { test } from "vitest";
const items = [];
test.apply(null, ["suppression", () => {
  // no-mistakes-disable-next-line no-mistakes/test-no-shared-state
  items.push(1);
  items.push(1); // no-mistakes-disable-line no-mistakes/test-no-shared-state
}]);

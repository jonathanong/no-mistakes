// no-mistakes-disable-file no-mistakes/test-no-shared-state
import { test } from "vitest";
const items = [];
test.apply(null, ["suppression", () => { items.push(1); }]);

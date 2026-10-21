import { test } from "@playwright/test";

const mixedDetails = {
  annotation: [{ type: "skip" }, { type: "fixme" }],
};

test("mixed skip and fixme declarations", mixedDetails, () => {});
const flag = true;
test(
  "allowed skip still reports a banned fixme branch",
  { annotation: { type: flag ? "skip" : "fixme" } },
  () => {},
);

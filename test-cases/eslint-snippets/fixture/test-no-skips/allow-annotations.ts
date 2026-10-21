import { test } from "@playwright/test";

const mixedDetails = {
  annotation: [{ type: "skip" }, { type: "fixme" }],
};

test("mixed skip and fixme declarations", mixedDetails, () => {});

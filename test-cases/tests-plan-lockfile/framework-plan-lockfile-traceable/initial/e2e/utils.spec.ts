import { utils } from "../src/utils";
import { test } from "@playwright/test";

test("uses utils", async () => {
  utils.pick({}, []);
});

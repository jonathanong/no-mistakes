import { test } from "@playwright/test";

test.slow(false);
(test.slow as typeof test.slow)(false);
test.slow();

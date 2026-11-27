import { test } from "@playwright/test";

test.setTimeout.bind(require("@playwright/test").test)(45000);

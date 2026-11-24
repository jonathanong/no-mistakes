import { test } from "@playwright/test";

test.describe.configure({ mode: "serial" });

test.describe.serial("ordered checkout", () => {});

// no-mistakes-disable-next-line playwright-no-serial
test.describe.serial("suppressed suite", () => {});

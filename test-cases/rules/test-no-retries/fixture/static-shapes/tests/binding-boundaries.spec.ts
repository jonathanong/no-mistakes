import type { PlaywrightTestConfig } from "@playwright/test";
import defaultRunner, { type TestType, test } from "@playwright/test";

export type RunnerSettings = PlaywrightTestConfig & TestType<object, object>;

defaultRunner.test.describe.configure({ mode: "serial", retries: 99 });
test.describe["\uD800"]({ mode: "serial", retries: 99 });

import { test as base } from "@playwright/test";
import { createOpaqueTest } from "../helpers/factory.mts";

const opaque = createOpaqueTest({ user: "guest" });
// no-mistakes-disable-next-line playwright-no-serial
opaque.describe.serial("ordered", () => {});
// no-mistakes-disable-next-line test-no-retries
opaque.describe.configure({ retries: 2 });

let local = base.extend({});
// no-mistakes-disable-next-line playwright-no-serial
local.describe.serial("ordered", () => {});
// no-mistakes-disable-next-line test-no-retries
local.describe.configure({ retries: 2 });

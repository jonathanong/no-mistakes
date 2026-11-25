// no-mistakes-disable-file playwright-no-serial
// no-mistakes-disable-file test-no-retries
import { createOpaqueTest } from "../helpers/factory.mts";

const opaque = createOpaqueTest({ user: "guest" });
opaque.describe.serial("ordered", () => {});
opaque.describe.configure({ retries: 2 });

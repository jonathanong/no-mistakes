import { createOpaqueTest, createTest } from "../helpers/factory.mts";

const followed = createTest();
followed.describe.serial("ordered", () => {});

const followedWithArgs = createTest({ user: "admin" });
followedWithArgs.describe.configure({ retries: 2 });

const opaque = createOpaqueTest({ user: "guest" });
opaque.describe.serial("ordered", () => {});
opaque.describe.configure({ retries: 2 });

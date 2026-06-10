import { test } from "@playwright/test";

// #397: the real-world shape — a module-scoped `let` initialized to "" and reassigned from
// another variable in `beforeAll` before being interpolated into a navigation path. The
// reassignment must not cause the initializer "" to leak into the matched path: the navigation
// must still edge the dynamic `[idOrUsername]` page and must NOT edge `/user/settings`.
const seededUserId = "seeded-id";
let targetUserId = "";

test.beforeAll(async () => {
  // Assigned at runtime; unresolvable to a concrete path at analysis time.
  targetUserId = seededUserId;
});

test("template literal nav with reassigned let", async ({ page }) => {
  await navigateTo(page, `/user/${targetUserId}`);
});

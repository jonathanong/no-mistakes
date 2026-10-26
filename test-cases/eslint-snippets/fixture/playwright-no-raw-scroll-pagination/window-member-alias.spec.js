import { test } from "@playwright/test";

// A cursor wait is present on purpose. These bindings stay silent because the
// destructured property is not scroll, scrollBy, or scrollTo. The local name
// is not the method name.
await page.waitForResponse((response) => response.url().includes("?cursor=1"));
const { alert: notify } = window;
notify("done");
const { requestAnimationFrame: raf } = window;
raf(() => {});
void test;

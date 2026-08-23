import { test } from "@playwright/test";

// Intentionally unrelated to the selector change — must NOT be selected when
// only `data-testid="old-selector"` is removed from `src/selector.tsx`.
test("unrelated", async () => {});

import { expect } from "@playwright/test";

expect(locator).toHaveClass("duration-300");
expect(locator.className).toContain("bg-primary");

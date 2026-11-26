declare const locator: { isVisible(): Promise<boolean> };

locator.isVisible().catch(() => false);
locator["isVisible" as const]().catch(() => false);
locator.isVisible()["catch" as const](() => false);

import "@playwright/test";

declare const page: any;

page.locator(".submit-btn");
page.locator(".submit-btn" as const);
page.locator(".submit-btn" satisfies string);
page.locator(<const>".submit-btn");
page.locator(".submit-btn"!);
(page.locator as any)(".submit-btn");
(<any>page.locator)(".submit-btn");
(page.locator<any>)(".submit-btn");
page["locator" as const](".submit-btn");
page.locator.apply(page, [".submit-btn"] as const);
page.locator('[data-pw="submit"]' as const);

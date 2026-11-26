declare const page: any;

page.locator('[data-testid="submit"]');
page.locator('[data-testid="submit"]' as const);
page.locator('[data-testid="submit"]' satisfies string);
page.locator(<const>'[data-testid="submit"]');
(page.locator as any)('[data-testid="submit"]');
(page.locator<any>)('[data-testid="submit"]');
page["locator" as const]('[data-testid="submit"]');
page.locator.apply(page, ['[data-testid="submit"]'] as const);
page.getByTestId("submit" as const);

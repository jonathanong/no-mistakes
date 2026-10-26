// A self-referential alias is unknown, even when named expect.
const expect = expect;
expect(locator).toBeVisible({ timeout: 20000 });

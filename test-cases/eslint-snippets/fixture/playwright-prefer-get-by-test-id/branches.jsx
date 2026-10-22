const { skipped, locator, ...rest } = page;
locator("[data-pw=save]");
const method = "locator";
const { [method]: hidden } = page;
hidden("[data-pw=save]");
const { locator: named = page.locator, ...extras } = page;
named("[data-pw=save]");
const [first] = [page.locator];
first("[data-pw=save]");
page.locator.call(...args, "[data-pw=save]");
page.locator.apply(page, ["[data-pw=save]"]?.concat());
page.click('[data-testid="save"]');
// no-mistakes-disable-next-line playwright-prefer-get-by-test-id
page.click('[data-testid="edit"]');

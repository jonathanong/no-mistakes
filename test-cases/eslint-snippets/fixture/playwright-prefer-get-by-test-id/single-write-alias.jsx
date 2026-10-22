let find = page.locator;
find('[data-pw="save"]');
var click = page.click;
click('[data-testid="edit"]');

// Reassignment fails closed even though the call still looks like a test-id selector.
let rebound = page.locator;
rebound = page.getByRole;
rebound('[data-pw="save"]');

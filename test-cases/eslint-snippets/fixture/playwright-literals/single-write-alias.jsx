let find = page.getByTestId;
find(`save-${user.id}`);
var read = page.getByTestId;
read(id);

// Reassignment fails closed: this call is not getByTestId.
let rebound = page.getByTestId;
rebound = page.locator;
rebound(`save-${user.id}`);

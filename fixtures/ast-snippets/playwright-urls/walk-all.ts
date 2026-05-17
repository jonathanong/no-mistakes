const setup = page.goto("/var-init");

function helper() {
  return page.goto("/return");
}

{
  page.goto("/block");
}

for (page.goto("/for-init-expr"); page.goto("/for-test"); page.goto("/for-update")) {
  page.goto("/for-body");
}

do {
  page.goto("/do-body");
} while (page.goto("/do-test"));

for (const key in page.goto("/for-in-right")) {
  page.goto("/for-in-body");
}

for (const value of page.goto("/for-of-right")) {
  page.goto("/for-of-body");
}

const arrow = () => {
  page.goto("/arrow");
};

const conditional = ready ? page.goto("/conditional") : page.goto("/alternate");
const logical = ready && page.goto("/logical");
page.goto("/sequence-one"), page.goto("/sequence-two");

expect(other).toHaveURL("/ignored-other-expect");
expect(page).toHaveURL(new URL("/ignored-url", base));
navigateTo(other, "/navigate-second");

import * as pw from "@playwright/test";

const ns = pw;
ns.test.describe.serial("ordered", () => {});
ns.test.describe.configure({ retries: 2 });

let alias = pw;
alias.test.describe.serial("mutable alias", () => {});

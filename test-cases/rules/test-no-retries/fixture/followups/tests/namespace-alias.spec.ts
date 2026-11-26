import * as pw from "@playwright/test";
import * as local from "./local-base";

const ns = pw;
ns.test.describe.serial("ordered", () => {});
ns.test.describe.configure({ retries: 2 });

let alias = pw;
alias.test.describe.serial("mutable alias", () => {});

let localAlias = local;
localAlias.base.describe.serial("mutable local namespace", () => {});

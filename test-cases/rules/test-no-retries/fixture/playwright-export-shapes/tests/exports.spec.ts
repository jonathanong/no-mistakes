import { test as base } from "@playwright/test";
import { test as relayed } from "../fixtures/relay";
import defaultFromStar, { test as starred } from "../fixtures/runner";
import { test as mixed } from "../fixtures/mixed";
import { test as opaque } from "../fixtures/opaque";
import { test as unavailable } from "../fixtures/unavailable";

const local = base.extend({});
local.describe.serial("ordered", () => {});
relayed.describe.configure({ retries: 2 });
starred.describe.configure({ mode: "serial" });

defaultFromStar.describe.configure({ retries: 3 });
mixed.describe.configure({ retries: 4 });
opaque.describe.configure({ retries: 5 });
unavailable.describe.configure({ retries: 6 });

relayed.only("focused", () => {});
relayed.describe.only("focused", () => {});
relayed.beforeEach.configure({ retries: 7 });
const bound = relayed.bind(null);
bound.describe.configure({ retries: 8 });

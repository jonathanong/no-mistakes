import { test } from "../helpers/browser.mts";
import { test as extended } from "../helpers/barrel";
import { test as fake } from "../helpers/fake";
import { test as cycle } from "../helpers/loop";
import type { test as typeOnly } from "../helpers/browser.mts";

test.describe.configure({ mode: "serial", retries: 2 });
extended.describe.configure({ mode: "serial", retries: 3 });
extended.describe.configure({ mode: "parallel", retries: 0 });
fake.describe.configure({ mode: "serial", retries: 4 });
cycle.describe.configure({ mode: "serial", retries: 5 });
typeOnly.describe.configure({ mode: "serial", retries: 6 });

function shadowed(test: typeof fake) {
  test.describe.configure({ mode: "serial", retries: 7 });
}

import * as fixtures from "../helpers/bridge";
fixtures.test.describe.serial.only("ordered suite", () => {});

import { test as ambiguous } from "../helpers/ambiguous";
ambiguous.describe.configure({ mode: "serial", retries: 8 });

import { test as invalidExtension } from "../helpers/invalid-extension";
invalidExtension.describe.configure({ mode: "serial", retries: 9 });

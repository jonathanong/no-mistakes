const pw = require("@playwright/test");
pw.test.setTimeout(999_999);

const { test: t } = require("@playwright/test");
t.setTimeout(999_999);

import eq = require("@playwright/test");
eq.test.setTimeout(999_999);

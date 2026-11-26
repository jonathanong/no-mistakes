import pw = require("@playwright/test");

pw.test.setTimeout(999_999);

namespace Tools {
  export const local = { test: { setTimeout: (_timeout: number) => {} } };
}
import local = Tools.local;
local.test.setTimeout(999_999);

declare const loc: unknown;
declare const exportName: string;

const { [exportName]: e } = require("@playwright/test");
e(loc).toBeVisible({ timeout: 30000 });

import { verify } from "./assertions";
verify(loc).toBeVisible({ timeout: 30000 });

import * as local from "./local-base";

let suite: typeof local.base;
suite = local.base.extend({});

suite.describe.serial("ordered", () => {});
suite.describe.configure({ retries: 2 });

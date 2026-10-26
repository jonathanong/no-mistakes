import { test } from "vitest";
import * as v from "vitest";
// A literal apply array carries the same sequential options as a direct call.
test.apply(null, ["apply", { sequential: true }, () => {}]);
v.describe["apply"](null, ["string", { ["sequential"]: true }, () => {}]);
v.it[`apply`]?.(null, ["template", { [`sequential`]: true }, () => {}]);
const run = test;
run.apply(null, ["alias", { sequential: true }, () => {}]);

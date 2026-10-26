import { test } from "vitest";
import v from "vitest";
declare const options: unknown[];
declare const key: string;
test.apply(null, options);
test.apply(null, ["false", { sequential: false }, () => {}]);
test[key](null, ["computed", { sequential: true }, () => {}]);
v.apply(null, ["default is not a runner", { sequential: true }, () => {}]);
function shadow(test) { test.apply(null, ["shadow", { sequential: true }, () => {}]); }
// no-mistakes-disable-next-line no-vitest-sequential -- static forwarding suppression
test.apply(null, ["suppressed", { sequential: true }, () => {}]);

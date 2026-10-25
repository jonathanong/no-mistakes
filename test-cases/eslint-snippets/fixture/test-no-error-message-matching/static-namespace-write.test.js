import * as pw from "@playwright/test";
const ns = pw;
// An attempted write elsewhere must not provide a stable alias proof.
function replace() { ns = other; }
ns.expect(err.message).toBe("unknown");

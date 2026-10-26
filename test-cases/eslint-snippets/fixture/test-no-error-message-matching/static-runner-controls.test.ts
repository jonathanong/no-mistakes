import { test as check } from "vitest";
import * as p from "@playwright/test";
function shadow(check) { check("shadow", () => { const matched = err.message === "x"; }); }
function shadowNamespace(p) { p.expect(err.message).toBe("shadow"); }
let run = check;
run = other;
run("mutable", () => { const matched = err.message === "x"; });
// no-mistakes-disable-next-line test-no-error-message-matching -- namespace suppression
p.expect(err.message).toBe("suppressed");

import { vi } from "vitest";
import * as runner from "vitest";
import vitest from "vitest";
declare const key: string;
function shadow(vi) { const v = vi; v.mock("./shadow"); }
function shadowNamespace(runner) { const { vi: v } = runner; v.mock("./shadow"); }
let mutable = vi;
mutable = other;
mutable.mock("./mutable");
const { [key]: opaque } = runner;
opaque.mock("./opaque");
vitest.mock("./default");
// no-mistakes-disable-next-line vitest-mock-test-file-naming -- static alias suppression
const v = vi;
// no-mistakes-disable-next-line vitest-mock-test-file-naming -- static alias suppression
v.mock("./suppressed");

// A declared callable has no initializer proving a mock origin.
declare const install: (source: string) => void;
install("./declared");

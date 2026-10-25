import * as unrelated from "./other";
import { helper } from "vitest";
import wrongDefault from "vitest";
import * as v from "vitest";
declare const key: string;
const local = {};
local.test("local", () => { const match = err.message === "x"; });
unknown.test("unknown", () => { const match = err.message === "x"; });
unrelated.test("other", () => { const match = err.message === "x"; });
wrongDefault.test("default", () => { const match = err.message === "x"; });
helper.test("non-namespace", () => { const match = err.message === "x"; });
function shadow(v) { v.test("shadow", () => { const match = err.message === "x"; }); }
let ns = v;
ns = other;
ns.test("mutable namespace", () => { const match = err.message === "x"; });
const { [key]: opaque } = v;
opaque("computed", () => { const match = err.message === "x"; });
v[key]("computed member", () => { const match = err.message === "x"; });

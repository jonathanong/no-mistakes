import eq = require("@example/api");
import * as ns from "@example/api";
import { banned } from "@example/other";

const { danger } = require("@example/api");
const mod = require("@example/api");
const name = "danger";

banned();
eq.danger();
ns.danger();
danger();
mod.danger();
ns[name]();
export const { danger: exported } = require("@example/api");
export { danger as reexported } from "@example/api";

declare function Foo(): void;
declare function safe(): void;
new (Foo<string>)();
const seen = safe;
(safe<string>)();

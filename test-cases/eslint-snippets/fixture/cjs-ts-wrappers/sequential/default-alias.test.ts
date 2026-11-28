import vt from "vitest";
import { default as named } from "vitest";

const test = vt;
const it = named;
test.sequential("alias of default", () => {});
test.test.sequential("member", () => {});
it.sequential("alias of named default", () => {});
it.test.sequential("named member", () => {});

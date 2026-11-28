import test from "vitest";
import { default as named } from "vitest";
test.sequential("direct default", () => {});
test.test.sequential("namespace still is", () => {});
named.test.sequential("named default", () => {});

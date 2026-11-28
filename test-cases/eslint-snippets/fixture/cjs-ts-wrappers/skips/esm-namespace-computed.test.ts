import * as v from "vitest";

const key = "test";
v["test"].skip("literal", () => {});
v[key].skip("const key", () => {});

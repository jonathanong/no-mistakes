const source = "vitest";
let vt = require(source);
vt.test.sequential("dynamic let", () => {});
var named = require(source);
(0, named.test.sequential)("comma", () => {});
(0, named.test.sequential).call(null, "comma call", () => {});

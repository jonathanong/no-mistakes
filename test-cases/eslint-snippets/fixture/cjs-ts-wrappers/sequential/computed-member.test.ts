const sdk = require("vitest");
const key = "test";
sdk[key].sequential("computed member", () => {});
let run = require("vitest")[key];
run = other;
run.sequential("reassigned computed", () => {});

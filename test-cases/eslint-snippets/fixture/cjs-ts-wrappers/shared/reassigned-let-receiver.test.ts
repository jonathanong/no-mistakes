declare const specifier: string;
let runner = require(specifier);
runner = { test() {} };
runner.test("case", () => {});

declare const spec: string;
const fs = require(spec);
fs.mkdtemp("/tmp/example-", () => {});
fs.mkdtempSync("/tmp/example-");

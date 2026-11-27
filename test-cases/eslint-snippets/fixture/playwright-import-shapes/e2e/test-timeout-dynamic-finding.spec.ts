declare const moduleName: string;

const pw = require(moduleName);
pw.test.setTimeout(999_999);

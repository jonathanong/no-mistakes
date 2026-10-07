declare function require(specifier: string): unknown;
const loader = require;
loader("./indirect");

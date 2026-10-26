// A redeclared local require is not the ambient loader.
var require = other;
var require;
const load = require;
load("./a.test");

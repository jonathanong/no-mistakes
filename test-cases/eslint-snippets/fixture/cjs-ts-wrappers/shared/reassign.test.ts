let { test: check } = require("vitest");
check = function local(_name, fn) { fn(); };
const bag = [];
check("case", () => { bag.push(1); });
let member = require("vitest").test;
member = function local(_name, fn) { fn(); };
const more = [];
member("case", () => { more.push(1); });

const check = require("vitest").test;
const bag = [];
check("case", () => { bag.push(1); });
const open = require("@playwright/test").test;
const pages = [];
open("page", () => { pages.push(1); });

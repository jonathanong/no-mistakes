const direct = require("./catalog");
const nested = require("./barrel").group;
const picked = require("./catalog").TABLES;
const alias = picked;
const dynamic = require(moduleName);
const returned = (() => require("./catalog"))();

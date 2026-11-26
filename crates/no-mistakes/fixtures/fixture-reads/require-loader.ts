consume(loadFixture("require-loader").label);
const mod = require("./loader");
consume(mod.loadFixture(id).size);

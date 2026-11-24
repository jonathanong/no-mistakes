const { createRequire } = require("module");
const { createRequire: makeReq } = require("node:module");
const mod = require("node:module");
const propFactory = require("node:module").createRequire;
const other = require("./other");

const load = createRequire(__filename);
const renamed = makeReq(__filename);
const viaNs = mod.createRequire(__filename);
const viaDirect = require("module").createRequire(__filename);
const viaProp = propFactory(__filename);
const ignored = other.createRequire(__filename);

load("../widget.cjs");
renamed.resolve("../item.cjs");
viaNs("../ns.cjs");
viaDirect("../direct.cjs");
viaProp("../prop.cjs");
ignored("../ignored.cjs");

function inside() {
  const { createRequire: innerMake } = require("module");
  const inner = innerMake(__filename);
  inner("../inner.cjs");
}
inside();

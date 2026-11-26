const R = require("react");
function A() {
  return R.use(Promise.resolve(1));
}

const { use: useVal } = require("react");
function B() {
  return useVal(Promise.resolve(1));
}

import Eq = require("react");
function C() {
  return Eq.use(Promise.resolve(1));
}

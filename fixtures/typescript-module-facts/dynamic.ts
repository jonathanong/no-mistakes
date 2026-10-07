const name = "./dynamic";
import(name);
require(name);
require();
function shadow(require: (name: string) => unknown) { return require("./shadowed"); }
eval("var unknown = 1");
import equals = require("./legacy");
export = equals;

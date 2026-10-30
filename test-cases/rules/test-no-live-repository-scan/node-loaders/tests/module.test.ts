import "../helpers/dormant";
const name = "node:fs";
module.require("node:fs"); // module-literal
module.require(name); // module-computed
module.require?.("node:fs"); // module-optional
module?.require("node:fs"); // module-optional-member
module["require"]("fs-extra"); // module-bracket
module.require("node:path"); // unrelated-module

function custom(module: { require(name: string): unknown }) {
  module.require("node:fs"); // shadowed-module
}
custom({ require: () => ({}) });

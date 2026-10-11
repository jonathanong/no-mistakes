"use strict";
const { staticPropertyName } = require("./vitest-timeout-bindings");

function memberName(node, context) {
  return staticPropertyName({ key: node.property, computed: node.computed }, context);
}
function pathStartsWith(copyPath, members) {
  return members.every((name, index) => copyPath[index].key === name);
}
function arrayPath(array, node, path, spread) {
  if (spread && !path.length) return path;
  const index = array.elements.indexOf(node);
  // An earlier opaque spread makes the copied child's runtime index uncertain.
  if (array.elements.slice(0, index).some((element) => element?.type === "SpreadElement"))
    return null;
  if (!spread) return [{ key: String(index), container: "array" }, ...path];
  // Numeric object keys do not prove the positions yielded by a custom iterator.
  if (path[0].container !== "array") return null;
  return [{ key: String(index + Number(path[0].key)), container: "array" }, ...path.slice(1)];
}
function wrappedCopyPath(parent, node, path, context) {
  if (parent.type === "Property") {
    const name = staticPropertyName(parent, context);
    return name === null ? null : [{ key: name, container: "object" }, ...path];
  }
  if (parent.type === "ArrayExpression" && node.type !== "SpreadElement")
    return arrayPath(parent, node, path, false);
  if (parent.type === "SpreadElement" && parent.parent.type === "ArrayExpression")
    return arrayPath(parent.parent, parent, path, true);
  if (parent.type === "SpreadElement" && parent.parent.type === "ObjectExpression" && path.length)
    return [{ ...path[0], container: "object" }, ...path.slice(1)];
  return path;
}
module.exports = { memberName, pathStartsWith, wrappedCopyPath };

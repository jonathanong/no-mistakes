"use strict";
const { staticPropertyName } = require("./vitest-timeout-bindings");
const { arrayLength } = require("./vitest-timeout-array-length");

function memberName(node, context) {
  return staticPropertyName({ key: node.property, computed: node.computed }, context);
}
function pathStartsWith(copyPath, members) {
  return members.every((name, index) => copyPath[index].key === name);
}
function arrayPath(array, node, path, spread, context) {
  if (spread && !path.length) return path;
  const index = array.elements.indexOf(node);
  let offset = 0;
  // Only proven array lengths establish offsets; custom and escaped iterators
  // can yield a different number of elements than their initializer suggests.
  for (const element of array.elements.slice(0, index)) {
    const length = element?.type === "SpreadElement" ? arrayLength(element.argument, context) : 1;
    if (length === null) return null;
    offset += length;
  }
  if (!spread) return [{ key: String(offset), container: "array" }, ...path];
  // Numeric object keys do not prove the positions yielded by a custom iterator.
  if (path[0].container !== "array") return null;
  return [{ key: String(offset + Number(path[0].key)), container: "array" }, ...path.slice(1)];
}
function wrappedCopyPath(parent, node, path, context) {
  if (parent.type === "Property") {
    const name = staticPropertyName(parent, context);
    return name === null ? null : [{ key: name, container: "object" }, ...path];
  }
  if (parent.type === "ArrayExpression" && node.type !== "SpreadElement")
    return arrayPath(parent, node, path, false, context);
  if (parent.type === "SpreadElement" && parent.parent.type === "ArrayExpression")
    return arrayPath(parent.parent, parent, path, true, context);
  if (parent.type === "SpreadElement" && parent.parent.type === "ObjectExpression" && path.length)
    return [{ ...path[0], container: "object" }, ...path.slice(1)];
  return path;
}
module.exports = { memberName, pathStartsWith, wrappedCopyPath };

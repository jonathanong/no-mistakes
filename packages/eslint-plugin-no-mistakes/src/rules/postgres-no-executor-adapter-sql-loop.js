"use strict";

// `for (const item of iterable)`: a const binding with no initializer whose value comes from
// the iterable. Its type or its iterable root is the proof; the binding itself cannot be written.
function loopBinding(def) {
  return Boolean(
    def?.type === "Variable" &&
    def.parent.kind === "const" &&
    def.node.id.type === "Identifier" &&
    def.parent.parent?.type === "ForOfStatement" &&
    def.parent.parent.left === def.parent,
  );
}

function loopIterable(def) {
  return loopBinding(def) ? def.parent.parent.right : null;
}

module.exports = { loopBinding, loopIterable };

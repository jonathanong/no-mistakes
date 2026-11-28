"use strict";

// `const [first] = list` keeps the element type without `undefined` unless
// noUncheckedIndexedAccess is on, so the binding is still an unchecked index.
function isArrayElement(decl, identifier, initType) {
  return (
    decl.id.elements.some((element) => element?.name === identifier.name) &&
    initType?.symbol?.name === "Array"
  );
}

module.exports = { isArrayElement };

"use strict";

// `const [first] = list` keeps the element type without `undefined` unless
// noUncheckedIndexedAccess is on, so the binding is still an unchecked index.
const ARRAY_SYMBOLS = new Set(["Array", "ReadonlyArray"]);

function isArrayElement(decl, identifier, initType) {
  return (
    decl.id.elements.some((element) => element?.name === identifier.name) &&
    ARRAY_SYMBOLS.has(initType?.symbol?.name)
  );
}

module.exports = { isArrayElement };

"use strict";

function arrayValue(node, evaluate) {
  const entries = [];
  let knownPrefixLength = 0;
  let positionsKnown = true;
  for (const element of node.elements) {
    if (!element) {
      entries.push({ value: { kind: "unknown", runtimeType: "undefined" }, origin: node });
      if (positionsKnown) knownPrefixLength++;
    } else if (element.type === "SpreadElement") {
      const spread = evaluate(element.argument);
      if (spread.kind === "array" && !spread.runtimeFunction) {
        entries.push(...spread.entries.map((entry) => ({ ...entry, origin: element })));
        if (positionsKnown) {
          knownPrefixLength += spread.knownPrefixLength;
          positionsKnown = spread.lengthKnown;
        }
      } else {
        entries.push({ value: { kind: "unknown" }, origin: element });
        positionsKnown = false;
      }
    } else {
      entries.push({ value: evaluate(element), origin: element });
      if (positionsKnown) knownPrefixLength++;
    }
  }
  return { kind: "array", entries, knownPrefixLength, lengthKnown: positionsKnown };
}

function concatenateArrays(left, right) {
  return {
    kind: "array",
    entries: [...left.entries, ...right.entries],
    knownPrefixLength: left.lengthKnown
      ? left.entries.length + right.knownPrefixLength
      : left.knownPrefixLength,
    lengthKnown: left.lengthKnown && right.lengthKnown,
  };
}

module.exports = { arrayValue, concatenateArrays };

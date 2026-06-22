"use strict";

function collectPatternNames(node, names = new Set()) {
  if (!node) return names;
  if (node.type === "Identifier") {
    names.add(node.name);
    return names;
  }
  const children =
    node.type === "ObjectPattern"
      ? node.properties.map((property) => property.value || property.argument)
      : node.type === "ArrayPattern"
        ? node.elements
        : node.type === "RestElement"
          ? [node.argument]
          : node.type === "AssignmentPattern"
            ? [node.left]
            : [];
  for (const child of children) collectPatternNames(child, names);
  return names;
}

module.exports = {
  collectPatternNames,
};
